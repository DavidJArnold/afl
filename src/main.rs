mod tipping;

fn main() {
    let season = tipping::squiggle::get_squiggle_season(2024, "david.14587@gmail.com");
    println!("{:?}", season);
    tipping::models::run_model();
}
