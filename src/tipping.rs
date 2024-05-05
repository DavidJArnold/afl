use serde::{Deserialize, Serialize};
use std::fmt;

pub mod models;
pub mod squiggle;

struct Match {
    home_team: String,
    away_team: String,
    date: chrono::NaiveDateTime,
    venue: Option<String>,
}

struct MatchResult {
    winning_team: Option<Team>,
    winning_margin: Option<i32>,
    draw: bool,
    home_team_won: bool,
    away_team_won: bool,
}

struct MatchPrediction {
    prediction: f32,
    pred_margin: i32,
}

struct Team {
    name: String,
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SquiggleMatch {
    abehinds: Option<i32>,
    agoals: Option<i32>,
    ascore: Option<i32>,
    ateam: Option<String>,
    ateamid: Option<i32>,
    hbehinds: Option<i32>,
    hgoals: Option<i32>,
    hscore: Option<i32>,
    hteam: Option<String>,
    hteamid: Option<i32>,
    complete: Option<i8>,
    date: String,
    id: i32,
    is_final: i32,
    is_grand_final: i32,
    localtime: String,
    round: i32,
    roundname: Option<String>,
    timestr: Option<String>,
    tz: String,
    unixtime: i64,
    updated: Option<String>,
    venue: Option<String>,
    winner: Option<String>,
    winnerteamid: Option<i32>,
    year: Option<i32>,
}
