use crate::tipping::{
    models::{glicko::GlickoModel, margin::MarginModel},
    MatchTipping, ModelPerformance,
};

/// Generates comprehensive analysis reports
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate a detailed model analysis report
    pub fn generate_model_analysis(model: &GlickoModel) -> ModelAnalysisReport {
        let mut teams: Vec<_> = model.model_stats.iter().collect();
        teams.sort_by(|(_, a), (_, b)| b.elo.partial_cmp(&a.elo).unwrap());
        
        let elos: Vec<f64> = teams.iter().map(|(_, stats)| stats.elo).collect();
        let mean_elo = elos.iter().sum::<f64>() / elos.len() as f64;
        let elo_std = (elos.iter().map(|x| (x - mean_elo).powi(2)).sum::<f64>() / elos.len() as f64).sqrt();
        
        ModelAnalysisReport {
            total_teams: teams.len(),
            highest_elo: elos[0],
            lowest_elo: *elos.last().unwrap(),
            mean_elo,
            elo_standard_deviation: elo_std,
            top_team: teams[0].0.clone(),
            bottom_team: teams.last().unwrap().0.clone(),
        }
    }
    
    /// Generate performance metrics report
    pub fn generate_performance_report(
        year: i32,
        performance: &ModelPerformance,
        margin_model: &MarginModel,
    ) -> PerformanceReport {
        let accuracy = performance.total as f64 / performance.num_games as f64;
        let mean_mae = performance.mae as f64 / performance.num_games as f64;
        
        PerformanceReport {
            year,
            total_games: performance.num_games,
            correct_predictions: performance.total,
            accuracy_rate: accuracy,
            mean_absolute_error: mean_mae,
            bits_score: performance.bits,
            first_round_margin_error: performance.error_margin,
            margin_model_k: margin_model.k,
        }
    }
    
    /// Generate tips analysis
    pub fn analyze_tips(tips: &[MatchTipping]) -> TipsAnalysisReport {
        if tips.is_empty() {
            return TipsAnalysisReport::empty();
        }
        
        let total_tips = tips.len();
        let home_wins = tips.iter().filter(|t| t.home_or_away_wins == 'H').count();
        let away_wins = total_tips - home_wins;
        
        let margins: Vec<u32> = tips.iter().map(|t| t.margin).collect();
        let confidences: Vec<f64> = tips.iter().map(|t| t.percent).collect();
        
        let mean_margin = margins.iter().sum::<u32>() as f64 / margins.len() as f64;
        let mean_confidence = confidences.iter().sum::<f64>() / confidences.len() as f64;
        
        let high_confidence_tips = tips.iter().filter(|t| t.percent > 75.0).count();
        
        TipsAnalysisReport {
            total_tips,
            home_wins,
            away_wins,
            home_win_percentage: home_wins as f64 / total_tips as f64,
            mean_predicted_margin: mean_margin,
            mean_confidence: mean_confidence,
            high_confidence_tips,
            high_confidence_percentage: high_confidence_tips as f64 / total_tips as f64,
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ModelAnalysisReport {
    pub total_teams: usize,
    pub highest_elo: f64,
    pub lowest_elo: f64,
    pub mean_elo: f64,
    pub elo_standard_deviation: f64,
    pub top_team: String,
    pub bottom_team: String,
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub year: i32,
    pub total_games: u32,
    pub correct_predictions: u32,
    pub accuracy_rate: f64,
    pub mean_absolute_error: f64,
    pub bits_score: f64,
    pub first_round_margin_error: i64,
    pub margin_model_k: f64,
}

#[derive(Debug)]
pub struct TipsAnalysisReport {
    pub total_tips: usize,
    pub home_wins: usize,
    pub away_wins: usize,
    pub home_win_percentage: f64,
    pub mean_predicted_margin: f64,
    pub mean_confidence: f64,
    pub high_confidence_tips: usize,
    pub high_confidence_percentage: f64,
}

impl TipsAnalysisReport {
    fn empty() -> Self {
        Self {
            total_tips: 0,
            home_wins: 0,
            away_wins: 0,
            home_win_percentage: 0.0,
            mean_predicted_margin: 0.0,
            mean_confidence: 0.0,
            high_confidence_tips: 0,
            high_confidence_percentage: 0.0,
        }
    }
}