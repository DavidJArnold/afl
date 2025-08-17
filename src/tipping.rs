use serde::{Deserialize, Serialize};

pub mod models;
pub mod squiggle;

#[derive(Debug)]
pub struct Match {
    home_team: String,
    away_team: String,
    date: chrono::NaiveDateTime,
    venue: Option<String>,
}

#[derive(Debug)]
pub struct MatchResult {
    pub winning_team: Option<Team>,
    pub winning_margin: Option<u32>,
    pub draw: bool,
    pub home_team_won: bool,
    pub away_team_won: bool,
}

pub struct MatchPrediction {
    pub prediction: f64,
    pub pred_margin: u32,
    pub home_team_win: bool,
}

pub struct MatchTipping {
    pub home_or_away_wins: char,
    pub winner: String,
    pub margin: u32,
    pub percent: f64,
    pub home_team_name: String,
    pub away_team_name: String,
}

// Display logic moved to presentation module

#[derive(Debug, Clone)]
pub struct Team {
    pub name: String,
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

// Display logic moved to presentation module

#[derive(Debug, Serialize, Deserialize)]
pub struct SquiggleMatch {
    pub abehinds: Option<i32>,
    pub agoals: Option<i32>,
    pub ascore: Option<i32>,
    pub ateam: Option<String>,
    pub ateamid: Option<i32>,
    pub hbehinds: Option<i32>,
    pub hgoals: Option<i32>,
    pub hscore: Option<i32>,
    pub hteam: Option<String>,
    pub hteamid: Option<i32>,
    pub complete: Option<i8>,
    pub date: String,
    pub id: i32,
    pub is_final: i32,
    pub is_grand_final: i32,
    pub localtime: String,
    pub round: i32,
    pub roundname: Option<String>,
    pub timestr: Option<String>,
    pub tz: String,
    pub unixtime: i64,
    pub updated: Option<String>,
    pub venue: Option<String>,
    pub winner: Option<String>,
    pub winnerteamid: Option<i32>,
    pub year: Option<i32>,
}

impl SquiggleMatch {
    pub fn get_match(&self) -> Match {
        Match {
            home_team: self.hteam.clone().unwrap(),
            away_team: self.ateam.clone().unwrap(),
            date: chrono::NaiveDateTime::parse_from_str(&self.localtime, "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            venue: self.venue.clone(),
        }
    }

    pub fn get_match_result(&self) -> MatchResult {
        let margin = if self.hscore == self.ascore {
            None
        } else {
            Some((self.hscore.unwrap() - self.ascore.unwrap()).unsigned_abs())
        };
        let winning_team = if self.ascore.unwrap() == self.hscore.unwrap() {
            None
        } else {
            Some(Team {
                name: self.winner.clone().unwrap(),
            })
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

pub struct ModelPerformance {
    pub total: u32,
    pub num_games: u32,
    pub error_margin: i64,
    pub mae: i64,
    pub bits: f64,
}
