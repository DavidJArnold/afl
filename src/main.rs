use std::{collections::HashMap, env};

use afl::{run_model, presentation::Presenter};
use chrono::Datelike;

#[tokio::main]
async fn main() {
    let current_date = chrono::Utc::now();
    let year = current_date.year();

    let email = env::var("AFL_USER_EMAIL").expect("AFL_USER_EMAIL environment variable not set.");

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

    let (model, margin_model, perf, tips) = run_model(year, None, Some(offsets), email).await;

    let presenter = Presenter::console();
    presenter.display_model_summary(&model);
    presenter.display_tips(&tips);
    presenter.display_performance_summary(year, &perf, &margin_model);
}
