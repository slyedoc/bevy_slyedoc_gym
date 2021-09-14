


// Policy gradient example.
// This is based mainly on tch-rs examples and of course OpenAI Gym
//
// Because tch-rs is not thread safe we have to limit bevy in how it can access our PolicyGradientModel
// Will be using bevy non_send resources and exclusive_system(thread_local_system)

// https://github.com/bevyengine/bevy/blob/main/examples/ecs/ecs_guide.rs

use bevy::prelude::*;
use tch::{
    nn::{self, OptimizerConfig},
    Kind, Tensor,
};

use crate::environment::*;

pub struct PolicyGradientModelPlugin;
impl Plugin for PolicyGradientModelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_non_send_resource::<PolicyGradientModel>()
            .add_system(step.exclusive_system());
    }
}

pub struct PolicyGradientModel {
    pub model: nn::Sequential,
    pub opt: nn::Optimizer<nn::Adam>,
    pub steps: Vec<EnvironmentState>,
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

        println!("observation_space: {:?}", observation_space);
        println!("action_space: {:?}", action_space);

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


fn step(world: &mut World) {
    let world_cell = world.cell();
    let mut pg = world_cell
        .get_non_send_mut::<PolicyGradientModel>()
        .unwrap();

    // Get env state and run it though our model giving us an action
    let env_state = world_cell
        .get_resource_mut::<EnvironmentState>()
        .unwrap()
        .to_owned();
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
    env_action.action = action as usize;

    
    println!("obs: {:?}, action: {}", env_state.observation, action );
    pg.steps.push(env_state);

    // Update our EnvironmentCounter
    let mut env_counter = world_cell.get_resource_mut::<EnvironmentCounter>().unwrap();
    env_counter.step += 1;

    if env_counter.step >= env_counter.step_max {
        // start new episode
    }
    // let step = env.step(action)?;
    //         steps.push(step.copy_with_obs(&obs));
    //         obs = if step.is_done { env.reset()? } else { step.obs };
    //         if step.is_done && steps.len() > 5000 {d
    //             break;
    //         }
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

// fn accumulate_rewards(steps: &[Step]) -> Vec<f64> {
//     let mut rewards: Vec<f64> = steps.iter().map(|s| s.reward).collect();
//     let mut acc_reward = 0f64;
//     for (i, reward) in rewards.iter_mut().enumerate().rev() {
//         if steps[i].is_done {
//             acc_reward = 0.0;
//         }
//         acc_reward += *reward;
//         *reward = acc_reward;
//     }
//     rewards
// }
