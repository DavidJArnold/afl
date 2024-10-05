use std::env;

use afl::run_model;

#[tokio::main]
async fn main() {
    let year = 2024;

    let email = env::var("AFL_USER_EMAIL").expect("AFL_USER_EMAIL environment variable not set.");
    let (model, margin_model, perf, tips) = run_model(year, None, &email).await;

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
