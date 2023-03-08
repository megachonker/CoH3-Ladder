use serde_json::Value;
use std::error::Error;

async fn getpage(number : u64) -> Result<(), Box<dyn Error>>{
    // https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=50&leaderboard_id=2130306&start=2551&sortBy=1&title=coh3
    let url = format!("https://coh3-api.reliclink.com/community/leaderboard/getleaderboard2?count=16&leaderboard_id=2130306&start={}&sortBy=1&title=coh3",number);
    let json_str = reqwest::get(url)
    .await?
    .text()
    .await?;
    
    //parse
    let v:Value = serde_json::from_str(json_str.as_str()).unwrap();

    //get number of player
    let size = v["statGroups"].as_array().unwrap().len();

    for test  in 0..size  {
        //bind variable
        let player = &v["statGroups"][test]["members"][0];
        let game = &v["leaderboardStats"][test];

        //access var
        print!("{} from {} ",player["alias"],player["country"]);
        println!("Win/Loss {}/{}",game["wins"],game["losses"]);
    }
    Ok(())
}

#[tokio::main]
async fn main()  {
    // getpage(1).await.unwrap();
    getpage(2551).await.unwrap();
}
