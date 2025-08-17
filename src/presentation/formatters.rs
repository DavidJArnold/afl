use super::OutputFormatter;
use crate::tipping::{
    models::{glicko::GlickoModel, margin::MarginModel},
    MatchTipping, ModelPerformance,
};
use std::collections::HashMap;

/// Console-based formatter for terminal output
pub struct ConsoleFormatter {
    show_detailed_stats: bool,
}

impl ConsoleFormatter {
    pub fn new() -> Self {
        Self {
            show_detailed_stats: true,
        }
    }
    
    pub fn simple() -> Self {
        Self {
            show_detailed_stats: false,
        }
    }
}

impl Default for ConsoleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for ConsoleFormatter {
    fn format_model_summary(&self, model: &GlickoModel) {
        println!("=== AFL Model Team Rankings ===");
        let mut teams: Vec<_> = model.model_stats.iter().collect();
        teams.sort_by(|(_, a), (_, b)| b.elo.partial_cmp(&a.elo).unwrap());
        
        for (rank, (team_name, stats)) in teams.iter().enumerate() {
            if self.show_detailed_stats {
                println!(
                    "{:2}. {:20} | ELO: {:7.1} | RD: {:5.1} | Vol: {:.3}",
                    rank + 1,
                    team_name,
                    stats.elo,
                    stats.rd,
                    stats.volatility
                );
            } else {
                println!("{:2}. {}: {:.1}", rank + 1, team_name, stats.elo);
            }
        }
        println!();
    }
    
    fn format_tips(&self, tips: &[MatchTipping]) {
        if tips.is_empty() {
            println!("No upcoming matches to tip.");
            return;
        }
        
        println!("=== Match Predictions ===");
        for tip in tips {
            println!(
                "({}) {} by {} pts ({:.1}%): {} v {}",
                tip.home_or_away_wins,
                tip.winner,
                tip.margin,
                tip.percent,
                tip.home_team_name,
                tip.away_team_name
            );
        }
        println!();
    }
    
    fn format_performance_summary(
        &self,
        year: i32,
        performance: &ModelPerformance,
        margin_model: &MarginModel,
    ) {
        println!("=== Performance Summary ===");
        
        let accuracy = performance.total as f32 / performance.num_games as f32 * 100.0;
        let mean_mae = performance.mae as f64 / performance.num_games as f64;
        
        println!(
            "{} Season Results:",
            year
        );
        println!(
            "  Correct Tips: {}/{} ({:.2}%)",
            performance.total,
            performance.num_games,
            accuracy
        );
        println!(
            "  First Round Margin Error: {}",
            performance.error_margin
        );
        println!(
            "  Mean Absolute Error: {:.2}",
            mean_mae
        );
        println!(
            "  Bits Score: {:.3}",
            performance.bits
        );
        println!(
            "  Final Margin Model k: {:.1}",
            margin_model.k
        );
        println!();
    }
    
    fn format_offsets(&self, offsets: &HashMap<String, f64>) {
        println!("=== Optimized Team Offsets ===");
        let mut sorted_offsets: Vec<_> = offsets.iter().collect();
        sorted_offsets.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        
        for (team, offset) in sorted_offsets {
            println!("{:20}: {:8.3}", team, offset);
        }
        println!();
    }
    
    fn show_progress(&self) {
        print!(".");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }
}

/// JSON formatter for structured output
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_model_summary(&self, model: &GlickoModel) {
        let mut teams: Vec<_> = model.model_stats.iter().collect();
        teams.sort_by(|(_, a), (_, b)| b.elo.partial_cmp(&a.elo).unwrap());
        
        println!("{{");
        println!("  \"model_summary\": {{");
        println!("    \"teams\": [");
        
        for (i, (team_name, stats)) in teams.iter().enumerate() {
            let comma = if i < teams.len() - 1 { "," } else { "" };
            println!(
                "      {{\"name\": \"{}\", \"elo\": {:.1}, \"rd\": {:.1}, \"volatility\": {:.3}}}{}",
                team_name, stats.elo, stats.rd, stats.volatility, comma
            );
        }
        
        println!("    ]");
        println!("  }}");
        println!("}}");
    }
    
    fn format_tips(&self, tips: &[MatchTipping]) {
        println!("{{");
        println!("  \"tips\": [");
        
        for (i, tip) in tips.iter().enumerate() {
            let comma = if i < tips.len() - 1 { "," } else { "" };
            println!(
                "    {{\"winner\": \"{}\", \"margin\": {}, \"confidence\": {:.1}, \"home\": \"{}\", \"away\": \"{}\"}}{}",
                tip.winner, tip.margin, tip.percent, tip.home_team_name, tip.away_team_name, comma
            );
        }
        
        println!("  ]");
        println!("}}");
    }
    
    fn format_performance_summary(
        &self,
        year: i32,
        performance: &ModelPerformance,
        margin_model: &MarginModel,
    ) {
        let accuracy = performance.total as f32 / performance.num_games as f32 * 100.0;
        let mean_mae = performance.mae as f64 / performance.num_games as f64;
        
        println!("{{");
        println!("  \"performance\": {{");
        println!("    \"year\": {},", year);
        println!("    \"correct_tips\": {},", performance.total);
        println!("    \"total_games\": {},", performance.num_games);
        println!("    \"accuracy_percent\": {:.2},", accuracy);
        println!("    \"first_round_margin_error\": {},", performance.error_margin);
        println!("    \"mean_absolute_error\": {:.2},", mean_mae);
        println!("    \"bits_score\": {:.3},", performance.bits);
        println!("    \"margin_model_k\": {:.1}", margin_model.k);
        println!("  }}");
        println!("}}");
    }
    
    fn format_offsets(&self, offsets: &HashMap<String, f64>) {
        println!("{{");
        println!("  \"offsets\": {{");
        
        let sorted_offsets: Vec<_> = offsets.iter().collect();
        for (i, (team, offset)) in sorted_offsets.iter().enumerate() {
            let comma = if i < sorted_offsets.len() - 1 { "," } else { "" };
            println!("    \"{}\": {:.3}{}", team, offset, comma);
        }
        
        println!("  }}");
        println!("}}");
    }
    
    fn show_progress(&self) {
        // JSON formatter doesn't show progress dots
    }
}