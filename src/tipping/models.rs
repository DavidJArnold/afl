use crate::tipping::models::glicko::update;

use self::glicko::GlickoModelInitParams;

use super::squiggle::{get_squiggle_season, get_squiggle_teams};

pub mod glicko;

pub fn run_model() {
    let year = 2024;
    let cache = "squiggle_cache";
    let user_agent = "david.14587@gmail.com";
    let warmup_matches = get_squiggle_season(year-1, user_agent, cache);
    let tipping_matches = get_squiggle_season(year, user_agent, cache);
    let teams = get_squiggle_teams(&warmup_matches);

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
}
