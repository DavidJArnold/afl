use serde::{Deserialize, Serialize};
use std::fmt;

pub mod models;
pub mod squiggle;

#[derive(Debug)]
struct Match {
    home_team: String,
    away_team: String,
    date: chrono::NaiveDateTime,
    venue: Option<String>,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

impl SquiggleMatch {
    fn get_match(&self) -> Match {
        Match {
            home_team: self.hteam.as_ref().unwrap().clone(),
            away_team: self.ateam.as_ref().unwrap().clone(),
            date: chrono::NaiveDateTime::parse_from_str(&self.localtime, "%Y-%m-%d %H:%M:%S").expect(&format!("{:?}", self.timestr)),
            venue: self.venue.clone(),
        }
    }

    fn get_match_result(&self) -> MatchResult {
        let margin = if self.hscore == self.ascore {
            None 
        } else {
            Some((self.hscore.unwrap() - self.ascore.unwrap()).abs())
        };
        let winning_team = if self.ascore.unwrap() == self.hscore.unwrap() {
            None
        } else {
            Some(Team{name: self.winner.as_ref().unwrap().to_string()})
        };

        MatchResult {
            winning_team,
            winning_margin: margin,
            away_team_won: self.hscore.unwrap() < self.ascore.unwrap(),
            home_team_won: self.hscore.unwrap() > self.ascore.unwrap(),
            draw: self.hscore.unwrap() == self.ascore.unwrap(),

        }
    }
}
