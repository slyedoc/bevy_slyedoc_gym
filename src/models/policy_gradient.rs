// Policy gradient example.
// This is based mainly on tch-rs examples and of course OpenAI Gym
use bevy::prelude::*;
use tch::{
    nn::{self, OptimizerConfig},
    Kind::{self, Float},
    Tensor,
};

use crate::{environment::*};
use super::MLModel;

pub struct PolicyGradientModelPlugin;
impl Plugin for PolicyGradientModelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_non_send_resource::<PolicyGradientModel>()
            .add_system_to_stage( CoreStage::Update, PolicyGradientModel::update_action.exclusive_system());
    }
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
            steps: vec![],
        }
    }
}

pub struct PolicyGradientModel {
    pub model: nn::Sequential,
    pub opt: nn::Optimizer<nn::Adam>,
    pub steps: Vec<EnvironmentState>,
}

impl MLModel for PolicyGradientModel {
    fn update_action(world: &mut World) {

        //println!("update_action");
        let world_cell = world.cell();
        let mut pg = world_cell
            .get_non_send_mut::<PolicyGradientModel>()
            .unwrap();

        // Get env state and run it though our model giving us an action
        let env_state = world_cell.get_resource_mut::<EnvironmentState>().unwrap();

        let action = tch::no_grad(|| {
            Tensor::of_slice(env_state.observation.as_slice())
                .unsqueeze(0)
                .apply(&pg.model)
                .softmax(1, Kind::Float)
                .multinomial(1, true)
        });
        let action = f32::from(action);

        // update EnvironmentAction so our env can take that action
        let mut env_action = world_cell.get_resource_mut::<EnvironmentAction>().unwrap();
        let old_action = env_action.action;
        env_action.action = action as usize;
        env_action.take = true;
    
        // Add new step for replay later
        let mut state = env_state.to_owned();
        state.action = old_action;
        pg.steps.push(state);
    
        let mut env_counter = world_cell.get_resource_mut::<EnvironmentCounter>().unwrap();
        if env_counter.step > env_counter.step_max {
            let sum_r: f32 = pg.steps.iter().map(|s| s.reward).sum();
            println!(
                "epoch: {:<3} episodes: {:<5} avg reward per episode: {:.2}",
                env_counter.epoch,
                env_counter.episode,
                sum_r / env_counter.episode as f32
            );
    
            pg.train();
            pg.steps.clear();
    
            env_counter.episode = 0;
            env_counter.step = 0;
            env_counter.epoch += 1;
        } else if env_state.is_done {
            env_counter.episode += 1;
        }
        env_counter.step += 1;
    }
}


impl PolicyGradientModel {
    // Train the model via policy gradient on the rollout data.
    fn train(&mut self) {
        let batch_size = self.steps.len() as i64;
        let actions: Vec<i64> = self.steps.iter().map(|s| s.action as i64).collect();
        let actions = Tensor::of_slice(&actions).unsqueeze(1);
        let rewards = self.accumulate_rewards();
        let rewards = Tensor::of_slice(&rewards).to_kind(Kind::Float);
        let action_mask =
            Tensor::zeros(&[batch_size, 2], tch::kind::FLOAT_CPU).scatter_value(1, &actions, 1.0);
        let obs: Vec<Tensor> = self
            .steps
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
        let mut rewards: Vec<f32> = self.steps.iter().map(|s| s.reward).collect();
        let mut acc_reward = 0f32;
        for (i, reward) in rewards.iter_mut().enumerate().rev() {
            if self.steps[i].is_done {
                acc_reward = 0.0;
            }
            acc_reward += *reward;
            *reward = acc_reward;
        }
        rewards
    }
}



// for epoch_idx in 0..50 {
//     let mut obs = env.reset();
//     let mut steps: Vec<Step> = vec![];
//     // Perform some rollouts with the current model.
//     loop {
//         let action = tch::no_grad(|| {
//             obs.unsqueeze(0)
//                 .apply(&model)
//                 .softmax(1, Float)
//                 .multinomial(1, true)
//         });
//         let action = i64::from(action);
//         let step = env.step(action)?;
//         steps.push(step.copy_with_obs(&obs));
//         obs = if step.is_done { env.reset()? } else { step.obs };
//         if step.is_done && steps.len() > 5000 {
//             break;
//         }
//     }
//     let sum_r: f64 = steps.iter().map(|s| s.reward).sum();
//     let episodes: i64 = steps.iter().map(|s| s.is_done as i64).sum();
//     println!(
//         "epoch: {:<3} episodes: {:<5} avg reward per episode: {:.2}",
//         epoch.current,
//         episodes,
//         sum_r / episodes as f64
//     );

//     // Train the model via policy gradient on the rollout data.
//     let batch_size = steps.len() as i64;
//     let actions: Vec<i64> = steps.iter().map(|s| s.action).collect();
//     let actions = Tensor::of_slice(&actions).unsqueeze(1);
//     let rewards = accumulate_rewards(&steps);
//     let rewards = Tensor::of_slice(&rewards).to_kind(Float);
//     let action_mask =
//         Tensor::zeros(&[batch_size, 2], tch::kind::FLOAT_CPU).scatter_value(1, &actions, 1.0);
//     let obs: Vec<Tensor> = steps.into_iter().map(|s| s.obs).collect();
//     let logits = Tensor::stack(&obs, 0).apply(&model);
//     let log_probs =
//         (action_mask * logits.log_softmax(1, Float)).sum_dim_intlist(&[1], false, Float);
//     let loss = -(rewards * log_probs).mean(Float);
//     self.opt.backward_step(&loss)

// epoch.current += 1;
// println!("epoch: {}", epoch.current);
