use std::{collections::HashMap, env};

use afl::{optimise, run_model, tipping::squiggle::{get_squiggle_season, get_squiggle_teams}};
use futures::executor::block_on;

#[tokio::main]
async fn main() {
    let year = 2024;
    let email = env::var("AFL_USER_EMAIL").expect("AFL_USER_EMAIL environment variable not set.");
    let cache = "optimise_cache".to_string();

    let matches = block_on(get_squiggle_season(year, email.clone(), cache));
    
    let teams: Vec<String> = Vec::from_iter(get_squiggle_teams(&matches));

    let offsets: HashMap<String, f64> = optimise(year, teams, email.clone());
    println!("{:?}", offsets);

    let (model, margin_model, perf, tips) = block_on(run_model(year, None, Some(offsets), email));

    println!("{model}");

    for tip in tips {
        println!("{tip}");
    }

    println!(
        "{year} score {} from {} games ({:.2}%), first round margin {}",
        perf.total,
        perf.num_games,
        perf.total as f32 / perf.num_games as f32 * 100.0,
        perf.error_margin,
    );
    let mean_mae = perf.mae as f64 / perf.num_games as f64;
    println!(
        "MAE: {} BITS: {} (final k={})",
        mean_mae, perf.bits, margin_model.k
    );
}
