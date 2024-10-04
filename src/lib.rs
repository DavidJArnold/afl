pub mod tipping;

use crate::tipping::models::glicko::GlickoModel;
use tipping::models::margin::MarginModel;

use std::collections::HashMap;
use tipping::models::glicko::{predict, update, GlickoModelInitParams};

use tipping::{
    squiggle::{get_squiggle_season, get_squiggle_teams},
    MatchTipping, ModelPerformance, SquiggleMatch,
};

fn tip_season(
    tipping_matches: Vec<SquiggleMatch>,
    mut model: GlickoModel,
    mut margin_model: MarginModel,
) -> (
    GlickoModel,
    MarginModel,
    ModelPerformance,
    Vec<MatchTipping>,
) {
    let mut total = 0;
    let mut num_games = 0;
    let mut error_margin = 0;
    let mut mae = 0;
    let mut bits = 0.0;
    let mut tips: Vec<MatchTipping> = vec![];
    for round in 0..tipping_matches.iter().map(|x| x.round).max().unwrap() + 1 {
        let round_matches = tipping_matches.iter().filter(|x| x.round == round);
        let round_over = round_matches
            .clone()
            .all(|x| x.timestr == Some("Full Time".to_string()));
        let round_started = round_matches.clone().any(|x| x.timestr.is_some());
        let mut first_game = true;
        for game in round_matches {
            let mut p = predict(&model, &game.get_match(), None);
            p.pred_margin = margin_model.predict(p.prediction.max(1f64 - p.prediction));

            let predicted_winner = if p.home_team_win {
                game.hteam.as_ref().unwrap()
            } else {
                game.ateam.as_ref().unwrap()
            };
            let correct = predicted_winner == game.winner.as_ref().unwrap_or(predicted_winner);
            let scaled_pred =
                ((p.prediction.max(1.0f64 - p.prediction) - 0.5) * 1.2 + 0.5).min(1.0);

            if game.timestr == Some("Full Time".to_string()) {
                let game_result = &game.get_match_result();
                model = update(model, &game.get_match(), game_result);
                margin_model.add_result(
                    scaled_pred,
                    game_result.winning_margin.unwrap_or(0),
                    correct,
                );

                if round_over {
                    num_games += 1;
                    if game_result.draw {
                        total += 1;
                        let pred_error = p.pred_margin as i64;
                        mae += pred_error;
                        bits += 1.0 + 0.5 * (scaled_pred * (1.0 - scaled_pred)).log(2f64);
                        if first_game {
                            error_margin += pred_error;
                        }
                    } else if correct {
                        total += 1;
                        let pred_error = (p.pred_margin as i64
                            - game_result.winning_margin.unwrap_or(0) as i64)
                            .abs();
                        mae += pred_error;
                        bits += 1.0 + scaled_pred.log(2f64);
                        if first_game {
                            error_margin += pred_error;
                        }
                    } else {
                        let pred_error = (p.pred_margin as i64
                            + game_result.winning_margin.unwrap_or(0) as i64)
                            .abs();
                        mae += pred_error;
                        bits += 1.0 + (1.0 - scaled_pred).log(2f64);
                        if first_game {
                            error_margin += pred_error;
                        }
                    };
                    if margin_model.data.probs.len() > 25 {
                        margin_model.update();
                    }
                    first_game = false;
                    continue;
                }
            }
            if !round_over || !round_started {
                let w = if p.prediction >= 0.5 { 'H' } else { 'A' };
                tips.push(MatchTipping {
                    home_or_away_wins: w,
                    winner: predicted_winner.to_string(),
                    margin: p.pred_margin,
                    percent: scaled_pred * 100.0,
                    home_team_name: game.hteam.as_ref().unwrap().to_string(),
                    away_team_name: game.ateam.as_ref().unwrap().to_string(),
                });
            }
        }
        if !round_started || !round_over {
            break;
        };
    }
    (
        model,
        margin_model,
        ModelPerformance {
            total,
            num_games,
            error_margin,
            mae,
            bits,
        },
        tips,
    )
}

pub fn run_model(
    year: i32,
    cache_name: Option<&str>,
    user_agent: &str,
) -> (
    GlickoModel,
    MarginModel,
    ModelPerformance,
    Vec<MatchTipping>,
) {
    let cache = cache_name.unwrap_or("squiggle_cache");
    let warmup_matches = get_squiggle_season(year - 1, user_agent, cache);
    let tipping_matches = get_squiggle_season(year, user_agent, cache);
    let teams = get_squiggle_teams(&warmup_matches);

    let mut offsets: HashMap<String, f64> = HashMap::new();
    offsets.insert("Richmond".to_string(), 0.001_694);
    offsets.insert("Brisbane Lions".to_string(), 10.483_391);
    offsets.insert("Collingwood".to_string(), 0.000_452);
    offsets.insert("North Melbourne".to_string(), 29.997_125);
    offsets.insert("Adelaide".to_string(), 15.514_055);
    offsets.insert("Port Adelaide".to_string(), 19.697_79);
    offsets.insert("Hawthorn".to_string(), 0.430_927);
    offsets.insert("Western Bulldogs".to_string(), 18.616_764);
    offsets.insert("St Kilda".to_string(), 7.428_024);
    offsets.insert("Greater Western Sydney".to_string(), 29.997_696);
    offsets.insert("West Coast".to_string(), 26.929_782);
    offsets.insert("Sydney".to_string(), 12.146_814);
    offsets.insert("Fremantle".to_string(), 15.826_724);
    offsets.insert("Melbourne".to_string(), 20.315_649);
    offsets.insert("Carlton".to_string(), 12.527_585);
    offsets.insert("Essendon".to_string(), 9.211_65);
    offsets.insert("Gold Coast".to_string(), 11.175_802);
    offsets.insert("Geelong".to_string(), 29.992_775);

    let params = GlickoModelInitParams {
        teams,
        starting_volatility: None,
        starting_rd: None,
        offsets: Some(offsets),
        scale_factor: None,
        starting_elo: None,
        volatility_constraint: None,
    };

    let mut model = GlickoModel::new(params);

    let margin_model = MarginModel::new(None);

    for game in warmup_matches {
        let match_obj = game.get_match();
        let match_result = game.get_match_result();
        {
            model = update(model, &match_obj, &match_result);
        }
    }

    tip_season(tipping_matches, model, margin_model)
}
