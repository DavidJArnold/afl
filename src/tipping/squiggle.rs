use std::collections::HashSet;

use request_cache::{create_connection, request};
use serde_json::Value;

use crate::tipping::SquiggleMatch;

async fn call_squiggle_season(year: i32, user_agent: &str, cache_session: &str) -> String {
    let conn = create_connection(cache_session);
    let url = format!("https://api.squiggle.com.au/?q=games;year={}", year);
    let resp = request(
        &conn,
        url,
        "GET".to_string(),
        86_400,
        Some(false),
        Some(user_agent),
    ).await;
    resp.response
}

pub async fn get_squiggle_season(year: i32, user_agent: &str, cache_session: &str) -> Vec<SquiggleMatch> {
    let body = call_squiggle_season(year, user_agent, cache_session).await;
    let v: Value = serde_json::from_str(&body).unwrap();
    serde_json::from_value(v.get("games").unwrap().clone()).unwrap()
}

pub fn get_squiggle_teams(squiggle_games: &Vec<SquiggleMatch>) -> HashSet<String> {
    let mut names = HashSet::new();
    for game in squiggle_games {
        if let Some(name) = &game.ateam {
            names.insert(name.clone());
        }
        if let Some(name) = &game.hteam {
            names.insert(name.clone());
        }
    }
    names
}
