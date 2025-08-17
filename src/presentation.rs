use crate::tipping::{
    models::{glicko::GlickoModel, margin::MarginModel},
    MatchTipping, ModelPerformance,
};
use std::collections::HashMap;

pub mod formatters;
pub mod reports;

pub use formatters::*;
pub use reports::*;

/// Main presentation controller that coordinates different output formats
/// Implements a strategy pattern for flexible output formatting
pub struct Presenter {
    pub formatter: Box<dyn OutputFormatter>,
}

impl Presenter {
    pub fn new(formatter: Box<dyn OutputFormatter>) -> Self {
        Self { formatter }
    }
    
    pub fn console() -> Self {
        Self::new(Box::new(ConsoleFormatter::new()))
    }
    
    pub fn display_model_summary(&self, model: &GlickoModel) {
        self.formatter.format_model_summary(model);
    }
    
    pub fn display_tips(&self, tips: &[MatchTipping]) {
        self.formatter.format_tips(tips);
    }
    
    pub fn display_performance_summary(
        &self,
        year: i32,
        performance: &ModelPerformance,
        margin_model: &MarginModel,
    ) {
        self.formatter.format_performance_summary(year, performance, margin_model);
    }
    
    pub fn display_offsets(&self, offsets: &HashMap<String, f64>) {
        self.formatter.format_offsets(offsets);
    }
    
    pub fn show_optimization_progress(&self) {
        self.formatter.show_progress();
    }
}

/// Trait for different output formats (console, JSON, etc.)
pub trait OutputFormatter {
    fn format_model_summary(&self, model: &GlickoModel);
    fn format_tips(&self, tips: &[MatchTipping]);
    fn format_performance_summary(&self, year: i32, performance: &ModelPerformance, margin_model: &MarginModel);
    fn format_offsets(&self, offsets: &HashMap<String, f64>);
    fn show_progress(&self);
}