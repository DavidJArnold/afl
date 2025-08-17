use std::{collections::HashMap, env};
use afl::{
    run_model, 
    presentation::{Presenter, JsonFormatter, ConsoleFormatter},
    tipping::models::glicko::{GlickoModel, GlickoModelInitParams},
};
use chrono::Datelike;

#[tokio::main]
async fn main() {
    // This example demonstrates different presentation formats
    println!("=== AFL Prediction Presentation Demo ===\n");
    
    let email = env::var("AFL_USER_EMAIL").unwrap_or_else(|_| "demo@example.com".to_string());
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    
    // Sample team offsets for demo
    let mut offsets: HashMap<String, f64> = HashMap::new();
    offsets.insert("Richmond".to_string(), 0.001_694);
    offsets.insert("Brisbane Lions".to_string(), 10.483_391);
    offsets.insert("Collingwood".to_string(), 0.000_452);
    
    println!("1. Console Format (Detailed):");
    println!("----------------------------");
    let console_presenter = Presenter::new(Box::new(ConsoleFormatter::new()));
    console_presenter.display_offsets(&offsets);
    
    println!("2. Console Format (Simple):");
    println!("---------------------------");
    let simple_presenter = Presenter::new(Box::new(ConsoleFormatter::simple()));
    
    // Create a simple demo model for display
    let demo_teams = std::collections::HashSet::from([
        "Richmond".to_string(),
        "Brisbane Lions".to_string(), 
        "Collingwood".to_string()
    ]);
    
    let params = GlickoModelInitParams {
        teams: demo_teams,
        starting_volatility: None,
        starting_rd: None,
        offsets: Some(offsets.clone()),
        scale_factor: None,
        starting_elo: None,
        volatility_constraint: None,
    };
    
    let demo_model = GlickoModel::new(params);
    simple_presenter.display_model_summary(&demo_model);
    
    println!("3. JSON Format:");
    println!("---------------");
    let json_presenter = Presenter::new(Box::new(JsonFormatter::new()));
    json_presenter.display_offsets(&offsets);
    
    println!("\n4. Model Analysis Report:");
    println!("-------------------------");
    let report = afl::presentation::ReportGenerator::generate_model_analysis(&demo_model);
    println!("Total Teams: {}", report.total_teams);
    println!("ELO Range: {:.1} - {:.1}", report.highest_elo, report.lowest_elo);
    println!("Mean ELO: {:.1} (std: {:.1})", report.mean_elo, report.elo_standard_deviation);
    println!("Top Team: {}", report.top_team);
    println!("Bottom Team: {}", report.bottom_team);
    
    println!("\n=== Demo Complete ===");
    println!("To see live data, run with AFL_USER_EMAIL environment variable set.");
}