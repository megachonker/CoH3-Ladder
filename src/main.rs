use bincode;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};
use textplots::{Chart, Plot, Shape};
use tokio::time;

struct Snap {
    instantaner: Vec<Player>,
    date: DateTime<Utc>,
}

#[macro_use]
extern crate log;

#[derive(Serialize, Debug, Deserialize)]
struct Player {
    name: String,
    steam_link: String,
    country: [char; 2],
    xp: u16,
    wermart_2v2: RankGame, //to vec list for getting history
                           //ad new faction and mod here
}
#[derive(Serialize, Debug, Deserialize)]
struct RankGame {
    rank: u16,
    elo: u16,
    win: u16,
    lose: u16,
    streak: i8,
    lastmatchdate: u64,
}

impl Player {
    fn display(&self) {
        print!("-------------------------------------------");
        println!("Name: {}", self.name);
        println!("Steam Link: {}", self.steam_link);
        println!("Country: {}", self.country.iter().collect::<String>());
        println!("XP: {}", self.xp);
        println!("2v2 Rank Game:");
        self.wermart_2v2.display();
    }

    fn display_summary(&self) {
        print!(
            "{} ({}), ",
            self.name,
            self.country.iter().collect::<String>()
        );
        println!(
            "2v2 Rank: {} Elo: {} Wins: {} Losses: {}",
            self.wermart_2v2.rank,
            self.wermart_2v2.elo,
            self.wermart_2v2.win,
            self.wermart_2v2.lose
        );
    }
}

impl RankGame {
    fn display(&self) {
        println!("Rank: {}", self.rank);
        println!("Elo: {}", self.elo);
        println!("Wins: {}", self.win);
        println!("Losses: {}", self.lose);
        println!("Streak: {}", self.streak);
        println!("Last Match Date: {}", self.lastmatchdate);
    }
}

async fn get_nb_player() -> Result<u64, Box<dyn Error>> {
    let json_str = reqwest::get("https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=1&leaderboard_id=2130306&start=1&sortBy=1&title=coh3")
    .await?
    .text()
    .await?;

    //parse
    let v: Value = serde_json::from_str(json_str.as_str()).unwrap();
    let azer = v["rankTotal"].as_u64().unwrap();
    Ok(azer)
}

async fn getpage(rank_offset: u64) -> Result<Vec<Player>, Box<dyn Error>> {
    let url = format!("https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=200&leaderboard_id=2130306&start={}&sortBy=1&title=coh3",rank_offset);
    let json_str = reqwest::get(url).await?.text().await?;

    //parse
    let v: Value = serde_json::from_str(json_str.as_str()).unwrap();

    //if get end of player
    if v["rankTotal"].as_u64() <= Some(rank_offset) {
        print!("===========================END=========================");
        ///////////////////////
        // STOP HERE PLEASE
        ///////////////////////
    }

    //get number of player
    let num_players = v["statGroups"].as_array().unwrap().len();

    //allocating the number of player
    let mut player_list: Vec<Player> = Vec::with_capacity(num_players); // Preallocate vector

    //match Player ID with Game ID
    for statgroups_indice in 0..num_players {
        //bind variable
        let user_id = &v["statGroups"][statgroups_indice]["id"];

        for laderboard_indice in 0..num_players {
            let statgroup_id = &v["leaderboardStats"][laderboard_indice]["statgroup_id"];

            if statgroup_id == user_id {
                let j_player = &v["statGroups"][statgroups_indice]["members"][0];
                let j_game = &v["leaderboardStats"][laderboard_indice];
                //init player
                player_list.push(Player {
                    name: j_player["alias"].to_string(),
                    steam_link: j_player["name"].to_string(),
                    country: j_player["country"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .collect::<Vec<char>>()
                        .try_into()
                        .unwrap(),
                    xp: j_player["xp"].as_u64().unwrap() as u16,
                    //init game
                    wermart_2v2: RankGame {
                        rank: (j_game["rank"].as_u64().unwrap() as u16),
                        elo: (j_game["rating"].as_u64().unwrap() as u16),
                        win: (j_game["wins"].as_u64().unwrap() as u16),
                        lose: (j_game["losses"].as_u64().unwrap() as u16),
                        streak: (j_game["streak"].as_i64().unwrap() as i8),
                        lastmatchdate: (j_game["lastmatchdate"].as_u64().unwrap()),
                    },
                });
                break;
            }
        }
    }
    Ok(player_list)
}

async fn get_all(start: u64) {
    let nb_player = get_nb_player().await.unwrap();
    info!("ther is {} player", nb_player);

    debug!("start paralele carving");
    let mut handles = Vec::new();

    for player_offset in (start..nb_player).step_by(200) {
        let handle = tokio::spawn(async move {
            trace!("get page from {}", player_offset);
            getpage(player_offset).await.unwrap() //erreur propager mal
        });
        handles.push(handle);
    }

    let mut all: Vec<Player> = Vec::new();
    debug!("wait for all tasks to complete");
    for handle in handles {
        all.extend(handle.await.unwrap());
    }

    debug!("sorting");
    all.sort_by_key(|player| player.wermart_2v2.elo);

    //crée le noms de fichier
    let now: DateTime<Utc> = Utc::now();
    let date_time_str = now.format("%Y-%m-%d_%H:%M").to_string();
    let file_name = format!("Coh3LadderV1_{}.bin", date_time_str);

    let file = File::create(file_name).unwrap();
    bincode::serialize_into(file, &all).unwrap();
}

fn listfilecompatible() -> Vec<String> {
    let re = Regex::new(r"Coh3LadderV1_\d{4}-\d{2}-\d{2}_\d{2}:\d{2}\.bin").unwrap();
    let mut files: Vec<String> = Vec::new();
    for entry in fs::read_dir(".").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
            if re.is_match(&file_name) {
                files.push(file_name);
            }
        }
    }
    files
}

fn loadfiles() -> Vec<Snap> {
    let mut allgame: Vec<Snap> = Vec::new();
    let filelist = listfilecompatible();
    for file in filelist {
        allgame.push(Snap {
            instantaner: bincode::deserialize_from(File::open(&file).unwrap()).unwrap(),
            date: Utc
                .datetime_from_str(&file, "Coh3LadderV1_%Y-%m-%d_%H:%M.bin")
                .unwrap(),
        });
    }
    for game in &allgame {
        info!("{} is loaded", game.date);
    }
    allgame
}

fn plot(snaps_by_mj: &HashMap<&str, Vec<(u16, DateTime<Utc>)>>, player_name: &str) {
    // Get the player's ranking history
    let player_history = snaps_by_mj.get(player_name).unwrap();

    // Create a vector of (x, y) points for the graph
    let points: Vec<(f32, f32)> = player_history
        .iter()
        .enumerate()
        .map(|(i, (elo, date))| (i as f32, *elo as f32))
        .collect();

    // Plot the graph using textplots
    Chart::new(200, 80, 0.0, player_history.len() as f32)
        .lineplot(&Shape::Lines(&points))
        .display();
}

fn hashmap(snaps: &Vec<Snap>) -> HashMap<&str, Vec<(u16, DateTime<Utc>)>> {
    let snaps_by_mj: HashMap<&str, Vec<(u16, DateTime<Utc>)>> = snaps
        .iter()
        .flat_map(|snap| {
            snap.instantaner
                .iter()
                .map(move |mj| (mj.name.as_str(), mj.wermart_2v2.elo, snap.date))
        })
        .fold(HashMap::new(), |mut map, (name, elo, date)| {
            map.entry(name).or_insert_with(Vec::new).push((elo, date));
            map
        });
    let mut sorted_snaps_by_mj = HashMap::new();
    for (name, snaps) in snaps_by_mj {
        let mut sorted_snaps = snaps;
        sorted_snaps.sort_by_key(|snap| snap.1); // sort by date
        sorted_snaps_by_mj.insert(name, sorted_snaps);
    }
    sorted_snaps_by_mj
}

fn search(snaps: &Vec<Snap>, name: String) -> String {
    let re = Regex::new(name.as_str()).unwrap();
    let mut retval = "NOTHING".to_string();
    for instant in snaps {
        if let Some(joueur) = instant
            .instantaner
            .iter()
            .find(|joueur| re.is_match(&joueur.name))
        {
            print!("on {} ", instant.date);
            joueur.display_summary();
            retval = joueur.name.clone();
        }
    }
    retval //return une errreur
}

#[tokio::main]
async fn main() {
    env_logger::init();

    //getting arg
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide at least one argument");
        return;
    }
    let arg = &args[1];
    let start: u64 = arg
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u64>()
        .unwrap_or(0);
    //doig gen un erreur si pas digit

    if start == 1 {
        loop {
            get_all(1).await;
            for number in 1..10 {
                info!("remaining: {}", 10 - number);
                sleep(Duration::from_secs(60));
            }
        }
    }
    if start == 30 {
        let filemap = loadfiles();
        let meshash: HashMap<&str, Vec<(u16, DateTime<Utc>)>> = hashmap(&filemap);
        if let Some(joueur) = filemap.last() {
            for a in &joueur.instantaner {
                println!("Joeur: {}", a.name);
                plot(&meshash, &a.name);
            }
        }
    }

    if start == 20 {
        let filemap = loadfiles();
        plot(
            &hashmap(&filemap),
            search(&filemap, args[2].to_string()).as_str(),
        );
    }

    if start == 10 {
        search(&loadfiles(), args[2].to_string());
    }
}
