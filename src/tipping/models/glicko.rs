use crate::tipping::{Match, MatchPrediction, MatchResult};
use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
};

#[derive(Debug, Clone)]
pub struct GlickoTeamStats {
    pub elo: f64,
    pub rd: f64,
    pub volatility: f64,
    pub offset: f64,
}

#[derive(Debug, Clone)]
pub struct GlickoModel {
    pub model_stats: HashMap<String, GlickoTeamStats>,
    pub model_params: GlickoModelParams,
}

#[derive(Debug)]
pub struct GlickoModelInitParams {
    pub teams: HashSet<String>,
    pub starting_rd: Option<f64>,
    pub starting_volatility: Option<f64>,
    pub offsets: Option<HashMap<String, f64>>,
    pub scale_factor: Option<f64>,
    pub volatility_constraint: Option<f64>,
    pub starting_elo: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct GlickoModelParams {
    pub teams: HashSet<String>,
    pub starting_rd: f64,
    pub starting_volatility: f64,
    pub offsets: HashMap<String, f64>,
    pub scale_factor: f64,
    pub volatility_constraint: f64,
    pub starting_elo: f64,
}

impl GlickoModel {
    pub fn new(params: GlickoModelInitParams) -> GlickoModel {
        const STARTING_ELO: f64 = 1500.0;
        const DEFAULT_STARTING_RD: f64 = 15.0;
        const DEFAULT_STARTING_VOLATILITY: f64 = 0.05;
        const DEFAULT_SCALE_FACTOR: f64 = 173.718;
        const VOLATILITY_CONSTRAINT: f64 = 0.1;

        let starting_rating_deviation: f64 = params.starting_rd.unwrap_or(DEFAULT_STARTING_RD);
        let starting_volatility: f64 = params
            .starting_volatility
            .unwrap_or(DEFAULT_STARTING_VOLATILITY);
        let offsets: HashMap<String, f64> = params.offsets.unwrap_or_default();
        let scale_factor: f64 = params.scale_factor.unwrap_or(DEFAULT_SCALE_FACTOR);
        let volatility_constraint: f64 = params
            .volatility_constraint
            .unwrap_or(VOLATILITY_CONSTRAINT);
        let starting_elo: f64 = params.starting_elo.unwrap_or(STARTING_ELO);

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

// Display logic moved to presentation module

pub fn predict(model: &GlickoModel, match_: &Match, scale: Option<f64>) -> MatchPrediction {
    let scale: f64 = scale.unwrap_or(2.0f64.sqrt());
    let h_team = &match_.home_team;
    let a_team = &match_.away_team;

    let mu_h = (model.model_stats.get(h_team).unwrap().elo
        + model.model_params.offsets.get(h_team).unwrap_or(&0.0)
        - model.model_params.starting_elo)
        / model.model_params.scale_factor;
    let mu_a = (model.model_stats.get(a_team).unwrap().elo - model.model_params.starting_elo)
        / model.model_params.scale_factor;
    let home_team_win_prob = (1.0 / (1.0 + (-scale * (mu_h - mu_a)).exp()))
        .min(0.99)
        .max(0.01);
    MatchPrediction {
        prediction: home_team_win_prob,
        pred_margin: 0,
        home_team_win: home_team_win_prob >= 0.5,
    }
}

pub fn update(mut model: GlickoModel, match_: &Match, match_result: &MatchResult) -> GlickoModel {
    let h_team = &match_.home_team;
    let a_team = &match_.away_team;
    let mut h_team_stats = model.model_stats.clone().get(h_team).unwrap().clone();
    let mut a_team_stats = model.model_stats.clone().get(a_team).unwrap().clone();

    let mut h_team_rating = (h_team_stats.elo
        + model.model_params.offsets.get(h_team).unwrap_or(&0.0)
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
        * (outcome - e_(h_team_rating, a_team_rating, a_team_rd_scaled));
    let d2 = a_team_rd_scaled.powi(2)
        * g_(h_team_rd_scaled)
        * (1.0 - outcome - e_(a_team_rating, h_team_rating, h_team_rd_scaled));

    h_team_rating += d1;
    a_team_rating += d2;

    h_team_stats.elo = h_team_rating * model.model_params.scale_factor
        + model.model_params.starting_elo
        - h_team_stats.offset;
    a_team_stats.elo =
        a_team_rating * model.model_params.scale_factor + model.model_params.starting_elo;

    h_team_stats.rd = h_team_rd_scaled * model.model_params.scale_factor;
    a_team_stats.rd = a_team_rd_scaled * model.model_params.scale_factor;

    model.model_stats.get_mut(h_team).unwrap().volatility = h_team_stats.volatility;
    model.model_stats.get_mut(h_team).unwrap().rd = h_team_stats.rd;
    model.model_stats.get_mut(h_team).unwrap().elo = h_team_stats.elo;

    model.model_stats.get_mut(a_team).unwrap().volatility = a_team_stats.volatility;
    model.model_stats.get_mut(a_team).unwrap().rd = a_team_stats.rd;
    model.model_stats.get_mut(a_team).unwrap().elo = a_team_stats.elo;
    model
}

#[allow(clippy::too_many_arguments)]
fn new_volatility(
    model_params: &GlickoModelParams,
    vol: f64,
    rd: f64,
    mu: f64,
    mu_j: f64,
    phi_j: f64,
    score: f64,
    v: f64,
) -> f64 {
    let a = vol.powi(2).ln();
    let eps = 0.000001;
    let mut var_a = a;

    let mut var_b: f64;
    let delta: f64 = delta_(score, mu, mu_j, phi_j);
    let tau = model_params.volatility_constraint;
    if delta.powi(2) > rd.powi(2) + v {
        var_b = (delta.powi(2) - rd.powi(2) - v).ln();
    } else {
        let mut k = 1;
        while f_(
            rd,
            a - k as f64 * tau.powi(2).sqrt(),
            delta,
            v,
            a,
            model_params.volatility_constraint,
        ) < 0.0
        {
            k += 1;
        }
        var_b = a - k as f64 * tau.powi(2).sqrt();
    }

    let mut f_a = f_(rd, var_a, delta, v, a, model_params.volatility_constraint);
    let mut f_b = f_(rd, var_b, delta, v, a, model_params.volatility_constraint);

    while (var_b - var_a).abs() > eps {
        let var_c = var_a + ((var_a - var_b) * f_a) / (f_b - f_a);
        let f_c = f_(rd, var_c, delta, v, a, model_params.volatility_constraint);

        if f_c * f_b < 0.0 {
            var_a = var_b;
            f_a = f_b;
        } else {
            f_a /= 2.0;
        }
        var_b = var_c;
        f_b = f_c;
    }
    (var_a / 2.0).exp()
}

fn f_(rd: f64, x: f64, delta: f64, v: f64, a: f64, vol_constraint: f64) -> f64 {
    let ex = x.exp();
    let num = ex * (delta.powi(2) - rd.powi(2) - v - ex);
    let den = 2.0 * ((rd.powi(2) + v + ex).powi(2));
    (num / den) - (x - a) / vol_constraint.powi(2)
}

fn g_(phi: f64) -> f64 {
    1.0 / (1.0 + 3.0 * phi.powi(2) / PI.powi(2)).sqrt()
}

fn e_(mu: f64, mu_j: f64, phi_j: f64) -> f64 {
    1.0 / (1.0 + (-g_(phi_j) * (mu - mu_j)).exp())
}

fn v_(mu: f64, mu_j: f64, phi_j: f64) -> f64 {
    1.0 / (g_(phi_j).powi(2) * e_(mu, mu_j, phi_j) * (1.0 - e_(mu, mu_j, phi_j)))
}

fn delta_(score: f64, mu: f64, mu_j: f64, phi_j: f64) -> f64 {
    v_(mu, mu_j, phi_j) * g_(phi_j) * (score - e_(mu, mu_j, phi_j))
}

#[cfg(test)]
mod tests {
    use crate::tipping::Team;

    use super::*;

    const TOLERANCE: f64 = 0.001;

    #[test]
    fn test_model() {
        let model_params = GlickoModelInitParams {
            teams: HashSet::from(["A".to_string(), "B".to_string()]),
            starting_rd: None,
            starting_volatility: None,
            starting_elo: None,
            offsets: None,
            scale_factor: None,
            volatility_constraint: None,
        };

        let mut model = GlickoModel::new(model_params);
        // Model display removed - use presentation module
        let match_ = Match {
            home_team: "A".to_string(),
            away_team: "B".to_string(),
            venue: None,
            date: chrono::NaiveDateTime::parse_from_str("2024-04-01 10:10:10", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        };
        let h_team = Team {
            name: "A".to_string(),
        };
        let a_team = Team {
            name: "B".to_string(),
        };
        let match_result = MatchResult {
            winning_team: Some(h_team.clone()),
            winning_margin: Some(0),
            draw: false,
            home_team_won: true,
            away_team_won: false,
        };
        model = update(model.clone(), &match_, &match_result);
        // Model display removed - use presentation module
        assert!(
            (model.model_stats.get(&h_team.name).unwrap().elo - 1500.8613081137828).abs()
                < TOLERANCE
        );
        assert!(
            (model.model_stats.get(&a_team.name).unwrap().elo - 1499.1386918862172).abs()
                < TOLERANCE
        );
        model = update(model.clone(), &match_, &match_result);
        // Model display removed - use presentation module
        assert!(
            (model.model_stats.get(&h_team.name).unwrap().elo - 1501.9303887754816).abs()
                < TOLERANCE
        );
        assert!(
            (model.model_stats.get(&a_team.name).unwrap().elo - 1498.0696112245184).abs()
                < TOLERANCE
        );
        model = update(model.clone(), &match_, &match_result);
        // Model display removed - use presentation module
        assert!(
            (model.model_stats.get(&h_team.name).unwrap().elo - 1503.2020226041004).abs()
                < TOLERANCE
        );
        assert!(
            (model.model_stats.get(&a_team.name).unwrap().elo - 1496.7979772958996).abs()
                < TOLERANCE
        );
        model = update(model.clone(), &match_, &match_result);
        // Model display removed - use presentation module
        assert!(
            (model.model_stats.get(&h_team.name).unwrap().elo - 1504.6700786004337).abs()
                < TOLERANCE
        );
        assert!(
            (model.model_stats.get(&a_team.name).unwrap().elo - 1495.3299213995663).abs()
                < TOLERANCE
        );
    }

    #[test]
    fn test_model_2() {
        let model_params = GlickoModelInitParams {
            teams: HashSet::from(["A".to_string(), "B".to_string()]),
            starting_rd: None,
            starting_volatility: None,
            starting_elo: None,
            offsets: None,
            scale_factor: None,
            volatility_constraint: None,
        };

        let mut model = GlickoModel::new(model_params);
        // Model display removed - use presentation module
        let match_ = Match {
            home_team: "A".to_string(),
            away_team: "B".to_string(),
            venue: None,
            date: chrono::NaiveDateTime::parse_from_str("2024-04-01 10:10:10", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
        };
        let h_team = Team {
            name: "A".to_string(),
        };
        let a_team = Team {
            name: "B".to_string(),
        };
        let match_result = MatchResult {
            winning_team: Some(h_team.clone()),
            winning_margin: Some(0),
            draw: false,
            home_team_won: true,
            away_team_won: false,
        };
        for _ in 0..70 {
            model = update(model.clone(), &match_, &match_result);
        }
        // Model display removed - use presentation module
        assert!(
            (model.model_stats.get(&h_team.name).unwrap().elo - 1691.9490472591813).abs()
                < TOLERANCE
        );
        assert!(
            (model.model_stats.get(&a_team.name).unwrap().elo - 1308.0509527408187).abs()
                < TOLERANCE
        );
    }
}
