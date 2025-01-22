use std::collections::HashMap;
use std::iter::zip;

use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::particleswarm::ParticleSwarm;
use futures::executor::block_on;

use crate::run_model;

#[derive(Clone)]
struct TotalScore {
    year: i32,
    user_agent: String,
    team_list: Vec<String>,
}

impl TotalScore {
    fn construct_offsets(&self, offsets: &Vec<f64>) -> HashMap<String, f64> {
        let mut offset_map = HashMap::new();
        for (x, z) in zip(self.team_list.iter(), offsets.iter()) {
            offset_map.insert(x.to_string(), *z);
        }
        offset_map
    }
}

impl CostFunction for TotalScore {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let offsets = self.construct_offsets(param);
        let (_, _, perf1, _) = block_on(run_model(self.year-1, None, Some(offsets.clone()), self.user_agent.clone()));
        let (_, _, perf2, _) = block_on(run_model(self.year-2, None, Some(offsets.clone()), self.user_agent.clone()));
        let (_, _, perf3, _) = block_on(run_model(self.year-3, None, Some(offsets.clone()), self.user_agent.clone()));
        println!(".");
        Ok(-(perf1.bits + perf2.bits + perf3.bits) as f64)
    }
}

pub fn optimise(year: i32, team_list: Vec<String>, user_agent: String) -> HashMap<String, f64> {
    let cost_function = TotalScore { year, user_agent, team_list };

    let lb: Vec<f64> = [0.0_f64; 18].to_vec();
    let ub: Vec<f64> = [30.0_f64; 18].to_vec();

    let solver = ParticleSwarm::new((lb, ub), 80);

    let res = Executor::new(cost_function.clone(), solver)
        .configure(|state| state.max_iters(1000))
        .run()
        .unwrap();

    cost_function.construct_offsets(&res.state.best_individual.unwrap().position)
}
