use argmin::{
    core::{CostFunction, Error, Executor},
    solver::brent::BrentOpt,
};

#[derive(Clone)]
pub struct MarginModel {
    pub data: MarginData,
    pub k: f64,
}

#[derive(Clone)]
pub struct MarginData {
    pub probs: Vec<f64>,
    margins: Vec<u32>,
    correct: Vec<bool>,
}

fn margin_formula(k: f64, prob: f64) -> f64 {
    (k * (prob - 0.5f64)).round()
}

fn calculate_margin_error(k: f64, probs: Vec<f64>, margins: Vec<u32>, correct: Vec<bool>) -> u32 {
    // accumulate error over a series of games
    let mut error: i32 = 0;
    for ((p, m), c) in probs.iter().zip(margins.iter()).zip(correct.iter()) {
        if *c {
            error += (margin_formula(k, *p) as i32 - (*m as i32)).abs();
        } else {
            error += (margin_formula(k, *p) as i32 + (*m as i32)).abs();
        };
    }
    error.try_into().unwrap()
}

impl CostFunction for MarginData {
    type Param = f64;
    type Output = f64;

    fn cost(&self, k: &Self::Param) -> Result<Self::Output, Error> {
        Ok(calculate_margin_error(
            *k,
            self.probs.clone(),
            self.margins.clone(),
            self.correct.clone(),
        ) as f64)
    }
}

impl MarginModel {
    pub fn new(k: Option<f64>) -> MarginModel {
        MarginModel {
            data: MarginData {
                probs: Vec::new(),
                margins: Vec::new(),
                correct: Vec::new(),
            },
            k: k.unwrap_or(232.0f64),
        }
    }

    pub fn add_result(&mut self, prob: f64, margin: u32, correct: bool) {
        assert!(prob >= 0.5f64);
        self.data.probs.push(prob);
        self.data.margins.push(margin);
        self.data.correct.push(correct);
    }

    pub fn update(&mut self) {
        let solver = BrentOpt::new(0.0, 250.0);

        let res = Executor::new(self.clone().data, solver).run().unwrap();

        self.k = res.state.param.unwrap()
    }

    pub fn predict(&self, prob: f64) -> u32 {
        margin_formula(self.k, prob) as u32
    }
}
