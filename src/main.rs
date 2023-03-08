extern crate serde_json;

use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct Profile {
    alias: String,
    country: String,
}

#[derive(Debug, Deserialize)]
struct Member {
    profile_id: i64,
    name: String,
    personal_statgroup_id: i64,
    xp: i64,
    level: i64,
    leaderboardregion_id: i64,
    #[serde(flatten)]
    profile: Profile,
}

#[derive(Debug, Deserialize)]
struct Group {
    id: i64,
    name: String,
    #[serde(rename = "type")]
    group_type: i64,
    members: Vec<Member>,
}

fn main() {
    let json_str = r#"[
        {"id":49316,"name":"","type":1,"members":[{"profile_id":17928,"name":"/steam/76561198092998893","alias":"vwvc","personal_statgroup_id":49316,"xp":781,"level":781,"leaderboardregion_id":2074437,"country":"cn"}]},
        {"id":96668,"name":"","type":1,"members":[{"profile_id":110912,"name":"/steam/76561198057483471","alias":"night_raven0203","personal_statgroup_id":96668,"xp":1431,"level":1431,"leaderboardregion_id":2074437,"country":"jp"}]}
    ]"#;

    let groups: Vec<Group> = serde_json::from_str(json_str).unwrap();

    for group in groups {
        for member in group.members {
            println!("Alias: {}, Country: {}", member.profile.alias, member.profile.country);
        }
    }
}
