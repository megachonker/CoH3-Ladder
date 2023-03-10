use serde_json::Value;
use std::env;
use std::error::Error;

#[macro_use]
extern crate log;

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

async fn getpage(rank_offset: u64) -> Result<bool, Box<dyn Error>> {
    // https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=50&leaderboard_id=2130306&start=2551&sortBy=1&title=coh3
    let url = format!("https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=200&leaderboard_id=2130306&start={}&sortBy=1&title=coh3",rank_offset);
    let json_str = reqwest::get(url).await?.text().await?;

    //parse
    let v: Value = serde_json::from_str(json_str.as_str()).unwrap();

    //if get end of player
    if v["rankTotal"].as_u64() <= Some(rank_offset) {
        print!("===========================END=========================");
        return Ok(false);
    }

    //get number of player
    let size = v["statGroups"].as_array().unwrap().len();

    //match Player ID with Game ID
    for statgroups_indice in 0..size {
        //bind variable
        let user_id = &v["statGroups"][statgroups_indice]["id"];

        for laderboard_indice in 0..size {
            let statgroup_id = &v["leaderboardStats"][laderboard_indice]["statgroup_id"];

            if statgroup_id == user_id{
                let player = &v["statGroups"][statgroups_indice]["members"][0];
                let game = &v["leaderboardStats"][laderboard_indice];
                print!(
                    "ELO {}: {} from {} ",
                    game["rating"], player["alias"], player["country"]
                );
                println!("Win/Loss {}/{}", game["wins"], game["losses"]);
            }
        }
    }
    Ok(true)
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
            getpage(player_offset).await.unwrap(); //erreur propager mal
        });
        handles.push(handle);
    }
    
    info!("wait for all tasks to complete");
    for handle in handles {
        handle.await.unwrap();
    }
}