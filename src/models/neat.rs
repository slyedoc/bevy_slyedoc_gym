// Neat example.
//
// https://github.com/suhdonghwi/neat

use std::time::Duration;

use crate::helpers;
use neat::innovation_record::InnovationRecord;
use neat::network::feedforward::Feedforward;
use neat::network::Network;
use neat::pool::Pool;

pub struct NeatML {
    innov_record: InnovationRecord,
    pub pool: Pool<Feedforward>,
    pub population: usize,
    generation_start: Duration,
    complete_agents: Vec<CompleteAgent>,
}

pub struct CompleteAgent {
    pub index: usize,
    pub fitness: f64,
}

impl NeatML {
    pub fn new(path: &str, start: Duration, verbosity: bool) -> Self {
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
            generation_start: start,
            complete_agents: Vec::new(),
        }
    }

    pub fn next_generation(&mut self, time: Duration) {
        let generation = self.pool.generation();


        if self.complete_agents.len() > 0 {
            // build up fitness list, and reset complete agents
            self.complete_agents.sort_by(|a, b| a.index.cmp(&b.index));
            let fitness_list: Vec<f64> = self
                .complete_agents
                .iter()
                .map(|a| a.fitness as f64)
                .collect();
            self.complete_agents.clear();

            let best_network = self
                .pool
                .evaluate(|i, network| network.evaluate(fitness_list[i]))
                .clone();
            let best_fitness = best_network.fitness().unwrap();

            println!(
                " best_fitness: {}, generation: {}",
                best_fitness, generation
            );

            self.pool.evolve(&mut self.innov_record);
            self.generation_start = time;
        }
    }

    pub fn record_complete_agent(&mut self, index: usize, time: Duration) {
        self.complete_agents.push(CompleteAgent {
            index: index,
            fitness: (time - self.generation_start).as_secs_f64(),
        })
    }
}
