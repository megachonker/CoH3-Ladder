use serde_json::Value;
use std::error::Error;
use std::env;
async fn getpage(rank_offset : u64) -> Result<bool, Box<dyn Error>>{

    
    // https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=50&leaderboard_id=2130306&start=2551&sortBy=1&title=coh3
    let url = format!("https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=200&leaderboard_id=2130306&start={}&sortBy=1&title=coh3",rank_offset);
    let json_str = reqwest::get(url)
    .await?
    .text()
    .await?;
    

    //parse
    let v:Value = serde_json::from_str(json_str.as_str()).unwrap();

    //if get end of player
    if v["rankTotal"].as_u64() <= Some(rank_offset)  {
        print!("===========================END=========================");
        return Ok(false);
    }

    //get number of player
    let size = v["statGroups"].as_array().unwrap().len();


    //match Player ID with Game ID
    for statgroups_indice  in 0..size  {
        
        //bind variable
        let user_id = &v["statGroups"][statgroups_indice]["id"];

        for laderboard_indice in 0..size  {
            let statgroup_id = &v["leaderboardStats"][laderboard_indice]["statgroup_id"];

            if statgroup_id == user_id{
                let player = &v["statGroups"][statgroups_indice]["members"][0];
                let game = &v["leaderboardStats"][laderboard_indice];
                print!("ELO {}: {} from {} ",game["rating"],player["alias"],player["country"]);
                println!("Win/Loss {}/{}",game["wins"],game["losses"]);
            }
        }
    }
    Ok(true)
}

#[tokio::main]
async fn main()  {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide at least one argument");
        return;
    }
    let arg = &args[1];
    let mut start:u64 = arg.chars().filter(|c| c.is_digit(10)).collect::<String>().parse::<u64>().unwrap_or(0);


    
    loop {
        println!("page number: {}", start);
        if let Ok(false) = getpage(start).await {
            return;
        }
        start+=200;
    }
    // getpage(start).await.unwrap();
}
