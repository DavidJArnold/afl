use std::collections::HashSet;

// use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use request_cache::{create_connection, request};
use serde_json::Value;

use crate::tipping::SquiggleMatch;

// pub fn get_squiggle_game() -> String {
//     Match {
//         home_team: "Adelaide".to_string(),
//         away_team: "Melbourne".to_string(),
//         date: NaiveDateTime::new(
//             NaiveDate::from_ymd_opt(2024, 4, 4).unwrap(),
//             NaiveTime::from_hms_opt(19, 40, 0).unwrap(),s
//         ),
//         venue: None,
//     };
//     let a_team = Team {
//         name: "Adelaide".to_string(),
//     };
//     println!("{}", a_team);
//     "A game".to_string()
// }

fn call_squiggle_season(year: i32, user_agent: &str, cache_session: &str) -> String {
    let conn = create_connection(cache_session);
    let url = format!("https://api.squiggle.com.au/?q=games;year={}", year);
    let resp = request(
        &conn,
        url,
        "GET".to_string(),
        86_400,
        Some(false),
        Some(user_agent),
    );
    resp.response
}

pub fn get_squiggle_season(year: i32, user_agent: &str, cache_session: &str) -> Vec<SquiggleMatch> {
    let body = call_squiggle_season(year, user_agent, cache_session);
    let v: Value = serde_json::from_str(&body).unwrap();
    serde_json::from_str(&v["games"].to_string()).unwrap()
}

pub fn get_squiggle_teams(squiggle_games: &Vec<SquiggleMatch>) -> HashSet<String> {
    let mut names = HashSet::new();
    for game in squiggle_games {
        if let Some(name) = &game.ateam {
            names.insert(name.to_string());
        }
        if let Some(name) = &game.hteam {
            names.insert(name.to_string());
        }
    }
    names
}
