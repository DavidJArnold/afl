use margin::MarginModel;

use self::glicko::GlickoModelInitParams;
use crate::tipping::models::glicko::{predict, update};
use std::collections::HashMap;

use super::squiggle::{get_squiggle_season, get_squiggle_teams};

pub mod glicko;
pub mod margin;

pub fn run_model(year: i32) {
    let cache = "squiggle_cache";
    let user_agent = "david.14587@gmail.com";
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

    let mut model = glicko::GlickoModel::new(params);

    let mut margin_model = MarginModel::new(None);

    for game in warmup_matches {
        let match_obj = game.get_match();
        let match_result = game.get_match_result();
        {
            model = update(model, &match_obj, &match_result);
        }
    }

    // println!("{model}");
    let mut total = 0;
    let mut num_games = 0;
    let mut error_margin = 0;
    let mut mae = 0;
    let mut bits = 0.0;
    for round in 0..tipping_matches.iter().map(|x| x.round).max().unwrap() + 1 {
        let round_matches = tipping_matches.iter().filter(|x| x.round == round);
        let round_over = round_matches
            .clone()
            .all(|x| x.timestr == Some("Full Time".to_string()));
        let round_started = round_matches.clone().any(|x| x.timestr.is_some());
        if round_started && !round_over {
            println!("{}", model);
        }
        if !round_started {
            break;
        }
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

            if game.timestr == Some("Full Time".to_string()) {
                let game_result = &game.get_match_result();
                model = update(model, &game.get_match(), game_result);
                margin_model.add_result(
                    p.prediction.max(1.0f64 - p.prediction),
                    game_result.winning_margin.unwrap_or(0),
                    correct,
                );

                if round_over {
                    num_games += 1;
                    if correct {
                        total += 1;
                        let pred_error = (p.pred_margin as i64 - game_result.winning_margin.unwrap_or(0) as i64).abs();
                        mae += pred_error;
                        bits += 1.0 + p.prediction.log(2f64);
                        if first_game {
                            error_margin += pred_error;
                        }
                    } else {
                        let pred_error = (p.pred_margin as i64 + game_result.winning_margin.unwrap_or(0) as i64).abs();
                        mae += pred_error;
                        bits += 1.0 + (1.0-p.prediction).log(2f64);
                        if first_game {
                            error_margin += pred_error;
                        }
                    };
                    margin_model.update();
                    first_game = false;
                    continue;
                }
            }
            if round_started && !round_over {
                println!(
                    "{} by {} pts ({}): {} v {}",
                    predicted_winner,
                    p.pred_margin,
                    p.prediction.max(1.0 - p.prediction),
                    &game.hteam.as_ref().unwrap(),
                    &game.ateam.as_ref().unwrap()
                );
            }
        }
    }
    println!("{year} score {total} from {num_games} games ({:.2}%), first round margin {error_margin}", total as f32 / num_games as f32 * 100.0);
    let mean_mae = mae as f64 / num_games as f64;
    println!("MAE: {mean_mae} BITS: {bits} (final k={})", margin_model.k);
}
