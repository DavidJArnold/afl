use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::Value;
use request_cache::{request, create_connection};

use crate::tipping::{SquiggleMatch, Match, Team};

pub fn get_squiggle_game() -> String {
    Match {
        home_team: "Adelaide".to_string(),
        away_team: "Melbourne".to_string(),
        date: NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 4).unwrap(),
            NaiveTime::from_hms_opt(19, 40, 0).unwrap(),
        ),
        venue: None,
    };
    let a_team = Team {
        name: "Adelaide".to_string(),
    };
    println!("{}", a_team);
    "A game".to_string()
}

fn call_squiggle_season(year: i32, user_agent: &str) -> String {
    let conn = create_connection("squiggle_cache");
    let url = format!(
            "https://api.squiggle.com.au/?q=games;year={}",
            year
        );
    let resp = request(&conn, url, "GET".to_string(), 86_400, Some(false), Some(user_agent));
    resp.response
}


pub fn get_squiggle_season(year: i32, user_agent: &str) -> Vec<SquiggleMatch> {
    let body = call_squiggle_season(year, user_agent);
    let v: Value = serde_json::from_str(&body).unwrap();
    serde_json::from_str(&v["games"].to_string()).unwrap()
}
