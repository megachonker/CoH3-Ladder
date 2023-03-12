use serde_json::Value;
use std::env;
use std::error::Error;
use serde::{Serialize};
use std::fs::File;
use std::io::prelude::*;


#[macro_use]
extern crate log;

#[derive(Serialize, Debug)]
struct Player {
    name: String,
    steam_link: String,
    country: [char; 2],
    xp: u64,
    wermart_2v2: RankGame, //to vec list for getting history
                           //ad new faction and mod here
}
#[derive(Serialize, Debug)]
struct RankGame {
    rank: u64,
    elo: u64,
    win: u64,
    lose: u64,
    streak: i64,
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
                    xp: j_player["xp"].as_u64().unwrap(),
                    //init game
                    wermart_2v2: RankGame {
                        rank: (j_game["rank"].as_u64().unwrap()),
                        elo: (j_game["rating"].as_u64().unwrap()),
                        win: (j_game["wins"].as_u64().unwrap()),
                        lose: (j_game["losses"].as_u64().unwrap()),
                        streak: (j_game["streak"].as_i64().unwrap()),
                        lastmatchdate: (j_game["lastmatchdate"].as_u64().unwrap()),
                    },
                });
                break;
            }
        }
    }
    Ok(player_list)
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

    let nb_player = get_nb_player().await.unwrap();
    info!("ther is {} player", nb_player);

    info!("start paralele carving");
    let mut handles = Vec::new();

    for player_offset in (start..nb_player).step_by(200) {
        let handle = tokio::spawn(async move {
            getpage(player_offset).await.unwrap() //erreur propager mal
        });
        handles.push(handle);
    }

    let mut all:Vec<Player> = Vec::new();
    info!("wait for all tasks to complete");
    for handle in handles {
        all.extend(handle.await.unwrap());
    }

    all.sort_by_key(|player| player.wermart_2v2.rank);

    for player in &all {
        player.display_summary();
    }
    
    let file = File::create("output.json").unwrap();
    serde_json::to_writer(file,&all).unwrap();

    // let deserialized:Vec<Player> = serde_json::from_reader(file).unwrap();


}
