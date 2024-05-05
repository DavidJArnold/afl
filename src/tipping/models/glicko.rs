use crate::tipping::{Match, MatchResult};
use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
    fmt,
};

#[derive(Debug, Clone)]
pub struct GlickoTeamStats {
    elo: f32,
    rd: f32,
    volatility: f32,
    offset: f32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
        let starting_volatility: f32 = params
            .starting_volatility
            .unwrap_or(DEFAULT_STARTING_VOLATILITY);
        let offsets: HashMap<String, f32> = params.offsets.unwrap_or_default();
        let scale_factor: f32 = params.scale_factor.unwrap_or(DEFAULT_SCALE_FACTOR);
        let volatility_constraint: f32 = params
            .volatility_constraint
            .unwrap_or(VOLATILITY_CONSTRAINT);
        let starting_elo: f32 = params.starting_elo.unwrap_or(STARTING_ELO);

        let mut model_stats = HashMap::new();
        for team in params.teams.clone().into_iter() {
            let team_stats = GlickoTeamStats {
                elo: STARTING_ELO,
                rd: starting_rating_deviation,
                volatility: starting_volatility,
                offset: *offsets.get(&team).unwrap_or(&0.0),
            };
            model_stats.insert(team, team_stats);
        }

        GlickoModel {
            model_stats,
            model_params: GlickoModelParams {
                teams: params.teams,
                starting_volatility,
                offsets,
                scale_factor,
                volatility_constraint,
                starting_rd: starting_rating_deviation,
                starting_elo,
            },
        }
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

    let mu_h = (model.model_stats.get(h_team).unwrap().elo
        + model.model_params.offsets.get(h_team).unwrap()
        - model.model_params.starting_elo)
        / model.model_params.scale_factor;
    let mu_a = (model.model_stats.get(a_team).unwrap().elo - model.model_params.starting_elo)
        / model.model_params.scale_factor;

    (1.0 / (1.0 + (-scale * (mu_h - mu_a)).exp()))
        .min(0.99)
        .max(0.01)
}

pub fn update(
    model: &mut GlickoModel,
    match_: Match,
    match_result: MatchResult,
) -> &mut GlickoModel {
    let h_team = match_.home_team;
    let a_team = match_.away_team;
    let mut h_team_stats = model.model_stats.clone().get(&h_team).unwrap().clone();
    let mut a_team_stats = model.model_stats.clone().get(&a_team).unwrap().clone();

    let mut h_team_rating = (h_team_stats.elo
        + model.model_params.offsets.get(&h_team).unwrap_or(&0.0)
        - model.model_params.starting_elo)
        / model.model_params.scale_factor;
    let mut a_team_rating =
        (a_team_stats.elo - model.model_params.starting_elo) / model.model_params.scale_factor;

    let mut h_team_rd_scaled = h_team_stats.rd / model.model_params.scale_factor;
    let mut a_team_rd_scaled = a_team_stats.rd / model.model_params.scale_factor;

    let v_h = v_(h_team_rating, a_team_rating, a_team_rd_scaled);
    let v_a = v_(a_team_rating, h_team_rating, h_team_rd_scaled);

    let outcome = if match_result.draw {
        0.5
    } else if match_result.home_team_won {
        1.0
    } else {
        0.0
    };

    h_team_stats.volatility = new_volatility(
        &model.model_params,
        h_team_stats.volatility,
        h_team_rd_scaled,
        h_team_rating,
        a_team_rating,
        a_team_rd_scaled,
        outcome,
        v_h,
    );
    a_team_stats.volatility = new_volatility(
        &model.model_params,
        a_team_stats.volatility,
        a_team_rd_scaled,
        a_team_rating,
        h_team_rating,
        h_team_rd_scaled,
        1.0 - outcome,
        v_a,
    );

    a_team_rd_scaled = 1.0
        / (1.0 / (a_team_rd_scaled.powi(2) + a_team_stats.volatility.powi(2)) + 1.0 / v_a).sqrt();
    h_team_rd_scaled = 1.0
        / (1.0 / (h_team_rd_scaled.powi(2) + h_team_stats.volatility.powi(2)) + 1.0 / v_h).sqrt();

    let d1 = h_team_rd_scaled.powi(2)
        * g_(a_team_rd_scaled)
        * (outcome - E_(h_team_rating, a_team_rating, a_team_rd_scaled));
    let d2 = a_team_rd_scaled.powi(2)
        * g_(h_team_rd_scaled)
        * (1.0 - outcome - E_(a_team_rating, h_team_rating, h_team_rd_scaled));

    h_team_rating += d1;
    a_team_rating += d2;

    h_team_stats.elo = h_team_rating * model.model_params.scale_factor
        + model.model_params.starting_elo
        - h_team_stats.offset;
    a_team_stats.elo =
        a_team_rating * model.model_params.scale_factor + model.model_params.starting_elo;

    h_team_stats.rd = h_team_rd_scaled * model.model_params.scale_factor;
    a_team_stats.rd = a_team_rd_scaled * model.model_params.scale_factor;

    model.model_stats.get_mut(&h_team).unwrap().volatility = h_team_stats.volatility;
    model.model_stats.get_mut(&h_team).unwrap().rd = h_team_stats.rd;
    model.model_stats.get_mut(&h_team).unwrap().elo = h_team_stats.elo;

    model.model_stats.get_mut(&a_team).unwrap().volatility = a_team_stats.volatility;
    model.model_stats.get_mut(&a_team).unwrap().rd = a_team_stats.rd;
    model.model_stats.get_mut(&a_team).unwrap().elo = a_team_stats.elo;

    model
}

fn new_volatility(
    model_params: &GlickoModelParams,
    vol: f32,
    rd: f32,
    mu: f32,
    mu_j: f32,
    phi_j: f32,
    score: f32,
    v: f32,
) -> f32 {
    let a = vol.powi(2).ln();
    let eps = 0.000001;
    let mut A = a;

    let mut B: f32 = 0.0;
    let delta: f32 = delta_(score, mu, mu_j, phi_j);
    let tau = model_params.volatility_constraint;
    if delta.powi(2) > rd.powi(2) + v {
        B = (delta.powi(2) - rd.powi(2) - v).ln();
    } else {
        let mut k = 1;
        while f_(
            rd,
            a - k as f32 * tau.abs(),
            delta,
            v,
            a,
            model_params.volatility_constraint,
        ) < 0.0
        {
            k += 1;
        }
        B = a - k as f32 * tau.abs();
    }

    let mut fA = f_(rd, A, delta, v, a, model_params.volatility_constraint);
    let mut fB = f_(rd, B, delta, v, a, model_params.volatility_constraint);

    while (B - A).abs() > eps {
        let C = A + ((A - B) * fA) / (fB - fA);
        let fC = f_(rd, C, delta, v, a, model_params.volatility_constraint);

        if fC * fB < 0.0 {
            A = B;
            fA = fB;
        } else {
            fA /= 2.0;
        }
        B = C;
        fB = fC;
    }
    (A / 2.0).exp()
}

fn f_(rd: f32, x: f32, delta: f32, v: f32, a: f32, vol_constraint: f32) -> f32 {
    let ex = x.exp();
    let num = ex * (delta.powi(2) - rd.powi(2) - v - ex);
    let den = 2.0 * ((rd.powi(2) + v + ex).powi(2));
    (num / den) - (x - a) / vol_constraint
}

fn g_(phi: f32) -> f32 {
    1.0 / (1.0 + 3.0 * phi.powi(2) / PI.powi(2))
}

fn E_(mu: f32, mu_j: f32, phi_j: f32) -> f32 {
    1.0 / (1.0 + (-g_(phi_j) * (mu - mu_j)).exp())
}

fn v_(mu: f32, mu_j: f32, phi_j: f32) -> f32 {
    1.0 / (g_(phi_j).powi(2) * E_(mu, mu_j, phi_j) * (1.0 - E_(mu, mu_j, phi_j)))
}

fn delta_(score: f32, mu: f32, mu_j: f32, phi_j: f32) -> f32 {
    v_(mu, mu_j, phi_j) * g_(phi_j) * (score - E_(mu, mu_j, phi_j))
}
