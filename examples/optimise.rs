use std::{collections::HashMap, env};

use afl::{optimise, run_model, presentation::Presenter, tipping::squiggle::{get_squiggle_season, get_squiggle_teams}};
use futures::executor::block_on;

#[tokio::main]
async fn main() {
    let year = 2024;
    let email = env::var("AFL_USER_EMAIL").expect("AFL_USER_EMAIL environment variable not set.");
    let cache = "optimise_cache".to_string();

    let matches = block_on(get_squiggle_season(year, email.clone(), cache));
    
    let teams: Vec<String> = Vec::from_iter(get_squiggle_teams(&matches));

    let offsets: HashMap<String, f64> = optimise(year, teams, email.clone());
    
    let presenter = Presenter::console();
    presenter.display_offsets(&offsets);

    let (model, margin_model, perf, tips) = block_on(run_model(year, None, Some(offsets), email));

    presenter.display_model_summary(&model);
    presenter.display_tips(&tips);
    presenter.display_performance_summary(year, &perf, &margin_model);
}
