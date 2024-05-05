use self::glicko::GlickoModelInitParams;

use super::squiggle::{get_squiggle_season, get_squiggle_teams};

pub mod glicko;

pub fn run_model() {
    let year = 2024;
    let matches = get_squiggle_season(year, "david.14587@gmail.com");
    let teams = get_squiggle_teams(matches);

    let params = GlickoModelInitParams {
        teams,
        starting_volatility: None,
        starting_rd: None,
        offsets: None,
        scale_factor: None,
        starting_elo: None,
        volatility_constraint: None,
    };

    let model = glicko::GlickoModel::new(params);
    println!("{:?}", model);
    println!("{}", model);
}
