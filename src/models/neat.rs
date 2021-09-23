// Neat example.
//
// https://github.com/suhdonghwi/neat

use std::time::Duration;

use crate::helpers;
use bevy::utils::HashMap;
use neat::innovation_record::InnovationRecord;
use neat::network::Network;
use neat::network::feedforward::Feedforward;
use neat::pool::Pool;

impl NeatML {
    pub fn new(path: &str, verbosity: bool) -> Self {
        let verbosity = match verbosity {
            true => 1,
            false => 0,
        };
        let params = helpers::read_parameters_file(path);
        let mut innov_record = InnovationRecord::new(params.input_number, params.output_number);
        let pool = Pool::<Feedforward>::new(params.clone(), verbosity, &mut innov_record);

        Self {
            innov_record: innov_record,
            pool: pool,
            population: params.population,
            generation_start: Duration::new(0, 0),
        }
    }
}



pub struct NeatML {
    innov_record: InnovationRecord,
    pool: Pool<Feedforward>,
    pub population: usize,
    generation_start: Duration,
}

impl NeatML {
    pub fn step(&mut self, observations: &[f32], reward: f32, done: bool) -> usize {
        let mut action: usize;
        let output = self
            .pool
            .activate_nth(
                0,
                &[
                    observations[0] as f64,
                    observations[1] as f64,
                    observations[2] as f64,
                ],
            )
            .unwrap();

        if output[0] > 0.5 {
            action = 0; // jump
        }
        action = 1; // nothing

        if done {
             let generation = self.pool.generation();
             let fitness_list: Vec<f64> = vec![reward as f64; 1];

             let mut best_network = self
                 .pool
                 .evaluate(|i, network| network.evaluate(fitness_list[i]))
                 .clone();
             let best_fitness = best_network.fitness().unwrap();

             println!(" best_fitness: {}, generation: {}", best_fitness, generation);

             self.pool.evolve(&mut self.innov_record);
        }
        println!("action: {}", action);
        action
    }
}
