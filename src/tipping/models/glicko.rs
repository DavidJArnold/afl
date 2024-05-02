use std::{collections::{HashMap, HashSet}, fmt};
use crate::tipping::Match;

#[derive(Debug)]
pub struct GlickoTeamStats {
    elo: f32,
    rd: f32,
    volatility: f32,
    offset: f32,
}

#[derive(Debug)]
pub struct GlickoModel {
    pub model_stats: HashMap<String, GlickoTeamStats>,
    pub model_params: GlickoModelParams,
}

#[derive(Debug)]
pub struct GlickoModelInitParams {
    pub teams: HashSet<String>,
    pub starting_rd: Option<f32>,
    pub starting_volatility: Option<f32>,
    pub offsets: Option<HashMap<String, f32>>,
    pub scale_factor: Option<f32>,
    pub volatility_constraint: Option<f32>,
    pub starting_elo: Option<f32>,
}

#[derive(Debug)]
pub struct GlickoModelParams {
    pub teams: HashSet<String>,
    pub starting_rd: f32,
    pub starting_volatility: f32,
    pub offsets: HashMap<String, f32>,
    pub scale_factor: f32,
    pub volatility_constraint: f32,
    pub starting_elo: f32,
}

impl GlickoModel {
    pub fn new(params: GlickoModelInitParams) -> GlickoModel {
        const STARTING_ELO: f32 = 1500.0;
        const DEFAULT_STARTING_RD: f32 = 15.0;
        const DEFAULT_STARTING_VOLATILITY: f32 = 0.05;
        const DEFAULT_SCALE_FACTOR: f32 = 173.718;
        const VOLATILITY_CONSTRAINT: f32 = 0.1;

        let starting_rating_deviation: f32 = params.starting_rd.unwrap_or(DEFAULT_STARTING_RD);
        let starting_volatility: f32 = params.starting_volatility.unwrap_or(DEFAULT_STARTING_VOLATILITY);
        let offsets: HashMap<String, f32> = params.offsets.unwrap_or(HashMap::<String, f32>::new());
        let scale_factor: f32 = params.scale_factor.unwrap_or(DEFAULT_SCALE_FACTOR);
        let volatility_constraint: f32 = params.volatility_constraint.unwrap_or(VOLATILITY_CONSTRAINT);
        let starting_elo: f32 = params.starting_elo.unwrap_or(STARTING_ELO);

        let mut model_stats = HashMap::new();
        for team in params.teams.clone().into_iter() {
            let team_stats = GlickoTeamStats { elo: STARTING_ELO, rd: starting_rating_deviation, volatility: starting_volatility, offset: *offsets.get(&team).unwrap_or(&0.0)};
            model_stats.insert(team, team_stats);
        }

        GlickoModel{model_stats, model_params: GlickoModelParams{ teams: params.teams, starting_volatility, offsets, scale_factor, volatility_constraint, starting_rd: starting_rating_deviation, starting_elo}}
    }
}

impl fmt::Display for GlickoModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut stats: Vec<(&String, &GlickoTeamStats)> = self.model_stats.iter().collect();
        stats.sort_by(|(_, a), (_, b)| b.elo.partial_cmp(&a.elo).unwrap());
        let mut output: Vec<String> = vec!["".to_string()];
        for team in &stats {
            output.push(format!("{}: {}", team.0, team.1.elo));
        }
        write!(f, "{}", output.join("\n"))
    }
}

pub fn predict(model: &GlickoModel, match_: &Match, scale: Option<f32>) -> f32 {
    let scale: f32 = scale.unwrap_or(2.0f32.sqrt());
    let h_team = &match_.home_team;
    let a_team = &match_.away_team;

    let mu_h = (model.model_stats.get(h_team).unwrap().elo + model.model_params.offsets.get(h_team).unwrap() - model.model_params.starting_elo) / model.model_params.scale_factor;
    let mu_a = (model.model_stats.get(a_team).unwrap().elo - model.model_params.starting_elo) / model.model_params.scale_factor;

    (1.0 / (1.0 + (-scale * (mu_h - mu_a)).exp())).min(0.99).max(0.01)
}

