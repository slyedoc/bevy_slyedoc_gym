// Policy gradient example.

// This is based mainly on tch-rs examples and of course OpenAI Gym
use bevy::prelude::*;
use tch::{
    nn::{self, OptimizerConfig},
    Kind::{self, Float},
    Tensor,
};

use crate::environment::*;

pub struct PolicyGradientModel {
    pub model: nn::Sequential,
    pub opt: nn::Optimizer<nn::Adam>,

    // History states
    // Reusing EnvironmentState, but this could use your own struct
    pub history: Vec<EnvironmentState>,

    // Few counters
    pub epoch: usize,
    pub epoch_max: usize,

    pub step: usize,
    pub step_max: usize,
}

impl FromWorld for PolicyGradientModel {
    fn from_world(world: &mut World) -> Self {
        let space = world
            .get_resource::<Environment>()
            .expect("Res<EnvSpace> not found.");

        let vs = nn::VarStore::new(tch::Device::Cpu);

        let p = &vs.root();
        let observation_space = space.observation_space as i64;
        let action_space = space.action_space as i64;

        println!("shape: in {}, out {}", observation_space, action_space);

        let model = nn::seq()
            .add(nn::linear(
                p / "lin1",
                observation_space,
                32,
                Default::default(),
            ))
            .add_fn(|xs| xs.tanh())
            .add(nn::linear(p / "lin2", 32, action_space, Default::default()));

        Self {
            model: model,
            opt: nn::Adam::default().build(&vs, 1e-2).unwrap(),
            history: vec![],

            epoch: 0,
            epoch_max: 50,
            step: 0,
            step_max: 5000,
        }
    }
}

impl PolicyGradientModel {
    
    pub fn step(&mut self, observations: &[f32], _reward: f32, done: Option<bool>) -> usize {

        //Use existing model to find an action
        let action = tch::no_grad(|| {
            Tensor::of_slice(observations)
                .unsqueeze(0)
                .apply(&self.model)
                .softmax(1, Kind::Float)
                .multinomial(1, true)
        });
        let action = f32::from(action) as usize;


        if self.step > self.step_max {
            let sum_r: f32 = self.history.iter().map(|s| s.reward ).sum();
            let episodes: f32 = self.history.iter().filter(|s| s.is_done.unwrap() ).count() as f32;

            println!(
                "epoch: {:<3} episodes: {:<5} avg reward per episode: {:.2}",
                self.epoch,
                episodes,
                sum_r / episodes
            );

            self.train();
            self.history.clear();
            
            self.step = 0;
            self.epoch += 1;
        }
        self.step += 1;

        action
    }

    fn train(&mut self) {
        let batch_size = self.history.len() as i64;
        let actions: Vec<i64> = self
            .history
            .iter()
            .map(|s| s.action.unwrap() as i64)
            .collect();
        let actions = Tensor::of_slice(&actions).unsqueeze(1);
        let rewards = Tensor::of_slice(&self.accumulate_rewards()).to_kind(Kind::Float);
        let action_mask =
            Tensor::zeros(&[batch_size, 2], tch::kind::FLOAT_CPU).scatter_value(1, &actions, 1.0);
        let obs: Vec<Tensor> = self
            .history
            .iter()
            .map(|s| Tensor::of_slice(&s.observation).to_kind(Kind::Float))
            .collect();
        let logits = Tensor::stack(&obs, 0).apply(&self.model);
        let log_probs =
            (action_mask * logits.log_softmax(1, Float)).sum_dim_intlist(&[1], false, Float);
        let loss = -(rewards * log_probs).mean(Float);
        self.opt.backward_step(&loss);
    }

    fn accumulate_rewards(&self) -> Vec<f32> {
        let mut rewards: Vec<f32> = self.history.iter().map(|s| s.reward).collect();
        let mut acc_reward = 0f32;
        for (i, reward) in rewards.iter_mut().enumerate().rev() {
            if self.history[i].is_done.unwrap() {
                acc_reward = 0.0;
             }
             acc_reward += *reward;
             *reward = acc_reward;
        }
        rewards
    }
}


