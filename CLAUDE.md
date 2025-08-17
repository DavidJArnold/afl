# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Environment Setup

This project requires the `AFL_USER_EMAIL` environment variable to be set, containing the user's email address. This is used as the User Agent for Squiggle API calls.

## Build and Run Commands

- **Build**: `cargo build` or `cargo build --release`
- **Run main binary**: `cargo run --bin run` (runs tipping predictions with hardcoded team offsets)
- **Run optimization example**: `cargo run --example optimise` (optimizes team offsets using particle swarm)
- **Test**: `cargo test`
- **Format**: `cargo fmt`
- **Lint**: `cargo clippy`

## Project Architecture

This is an AFL (Australian Football League) tipping prediction system that uses a Glicko rating model combined with margin prediction. The codebase is structured around:

### Core Components

- **`src/lib.rs`**: Main library entry point with `run_model()` function that orchestrates the tipping pipeline
- **`src/main.rs`**: Binary that runs predictions with hardcoded team offsets 
- **`src/tipping/`**: Core tipping functionality
  - **`models/glicko.rs`**: Enhanced Glicko rating system with venue-specific modeling, recent form tracking, and momentum indicators
  - **`models/margin.rs`**: Margin prediction model
  - **`squiggle.rs`**: API integration with Squiggle sports data service
- **`src/optimise/`**: Particle swarm optimization for finding optimal team offset parameters

### Data Flow

1. **Data Acquisition**: Fetches AFL match data from Squiggle API with caching via `request_cache`
2. **Model Training**: Trains Glicko model on previous season's data 
3. **Prediction Pipeline**: For each round, predicts match outcomes and margins, then updates models with actual results
4. **Optimization**: Uses particle swarm algorithm to optimize team-specific offset parameters across multiple seasons

### Key Types

- `SquiggleMatch`: Raw match data from API
- `Match`/`MatchResult`: Internal match representation
- `GlickoModel`: Enhanced team rating system with venue-specific advantages, recent form, and momentum tracking
- `GlickoTeamStats`: Team statistics including ELO, recent form, rolling averages, and momentum indicators
- `VenuePerformance`: Tracks team performance at specific venues
- `MarginModel`: Predicts winning margins
- `MatchTipping`: Final tip output with winner, margin, and confidence

### Caching Strategy

The system uses two cache directories:
- `squiggle_cache`: For regular match data (8-hour TTL)
- `optimise_cache`: For optimization runs to avoid repeated API calls

## Enhanced Prediction Features (Phase 1 Implementation)

### Venue-Specific Modeling
- **Venue Advantages**: Tracks and learns home ground advantages for each venue
- **Team-Venue Performance**: Monitors team-specific performance at different venues
- **Venue Impact**: Applies venue-specific adjustments to predictions

### Recent Form & Momentum
- **Time-Decay Weighting**: Recent matches weighted more heavily than older ones (default factor: 0.95)
- **Rolling Averages**: Tracks 5-game and 10-game rolling averages for each team
- **Momentum Indicators**: Measures trend in recent performance vs earlier results
- **Recent Form Integration**: Exponentially weighted recent performance affects predictions

### Enhanced Team Statistics
- **Games Count**: Tracks total games played for statistical significance
- **Rolling Performance**: Short-term (5 games) and medium-term (10 games) averages
- **Momentum Tracking**: Difference between recent and earlier performance trends
- **Venue-Specific Stats**: Win rates and margins at each venue

## Notable Dependencies

- **argmin**: Optimization framework used for particle swarm algorithm
- **request_cache**: Custom caching layer for API requests  
- **chrono**: Date/time handling for match scheduling
- **tokio**: Async runtime for API calls
- **serde**: JSON serialization for Squiggle API data