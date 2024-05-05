use self::glicko::GlickoModelInitParams;
use crate::tipping::models::glicko::{predict, update};
use std::collections::HashMap;

use super::squiggle::{get_squiggle_season, get_squiggle_teams};

pub mod glicko;

pub fn run_model() {
    let year = 2024;
    let cache = "squiggle_cache";
    let user_agent = "david.14587@gmail.com";
    let warmup_matches = get_squiggle_season(year - 1, user_agent, cache);
    let tipping_matches = get_squiggle_season(year, user_agent, cache);
    let teams = get_squiggle_teams(&warmup_matches);

    let mut offsets: HashMap<String, f32> = HashMap::new();
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
        offsets: None,
        scale_factor: None,
        starting_elo: None,
        volatility_constraint: None,
    };

    let mut model = glicko::GlickoModel::new(params);

    for game in warmup_matches {
        let match_obj = game.get_match();
        let match_result = game.get_match_result();
        update(&mut model, match_obj, match_result);
    }

    println!("{model}");
    let mut total = 0;
    let mut num_games = 0;
    for round in 0..tipping_matches.iter().map(|x| x.round).max().unwrap() + 1 {
        let round_matches = tipping_matches.iter().filter(|x| x.round == round);
        let round_over = round_matches
            .clone()
            .all(|x| x.timestr == Some("Full Time".to_string()));
        let round_started = round_matches.clone().any(|x| x.timestr.is_some());
        if !round_started {
            break;
        }
        for game in round_matches {
            let p = predict(&model, &game.get_match(), None);
            let predicted_winner = if p.home_team_win {
                game.hteam.as_ref().unwrap()
            } else {
                game.ateam.as_ref().unwrap()
            };
            if round_over {
                num_games += 1;
                if game.timestr == Some("Full Time".to_string())
                    && predicted_winner == game.winner.as_ref().unwrap_or(predicted_winner)
                {
                    total += 1;
                };
                continue;
            }
            if round_started && !round_over {
                println!(
                    "{} ({}): {} v {}",
                    predicted_winner,
                    p.prediction.max(1.0 - p.prediction),
                    &game.hteam.as_ref().unwrap(),
                    &game.ateam.as_ref().unwrap()
                );
            }
            if game.timestr == Some("Full Time".to_string()) {
                update(&mut model, game.get_match(), game.get_match_result());
            }
        }
    }
    println!("{total} ({num_games})");
}
