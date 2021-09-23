// Policy gradient example.

// This is based mainly on tch-rs examples and of course OpenAI Gym
use tch::{
    nn::{self, OptimizerConfig},
    Kind::{self, Float},
    Tensor,
};

pub struct PolicyGradientModel {
    pub model: nn::Sequential,
    pub opt: nn::Optimizer<nn::Adam>,
    history: Vec<HistoryState>,
}

struct HistoryState {
    pub reward: f32,
    pub is_done: bool,
    pub action: f32,
    pub observations: Vec<f32>,
}

impl PolicyGradientModel {
    pub fn new(input_size: i64, output: i64) -> Self {

        let vs = nn::VarStore::new(tch::Device::Cpu);
        let p = &vs.root();

        let model = nn::seq()
            .add(nn::linear(
                p / "lin1",
                input_size,
                32,
                Default::default(),
            ))
            .add_fn(|xs| xs.tanh())
            .add(nn::linear(p / "lin2", 32, output, Default::default()));

        Self {
            model: model,
            opt: nn::Adam::default().build(&vs, 1e-2).unwrap(),
            history: vec![],
        }
    }

    pub fn record_history(&mut self, observations: Vec<f32>, reward: f32, is_done: bool, action: f32) {
        self.history.push(HistoryState {
            reward: reward,
            is_done: is_done,
            action: action,
            observations: observations,
        });
    }

    pub fn train(&mut self) {
        let batch_size = self.history.len() as i64;
        let actions: Vec<i64> = self
            .history
            .iter()
            .map(|s| s.action as i64)
            .collect();
        let actions = Tensor::of_slice(&actions).unsqueeze(1);
        let rewards = Tensor::of_slice(&self.accumulate_rewards()).to_kind(Kind::Float);
        let action_mask =
            Tensor::zeros(&[batch_size, 2], tch::kind::FLOAT_CPU).scatter_value(1, &actions, 1.0);
        let obs: Vec<Tensor> = self
            .history
            .iter()
            .map(|s| Tensor::of_slice(&s.observations).to_kind(Kind::Float))
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
            if self.history[i].is_done {
                acc_reward = 0.0;
             }
             acc_reward += *reward;
             *reward = acc_reward;
        }
        rewards
    }
}


