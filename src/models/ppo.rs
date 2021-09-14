// TODO: PPO

// https://towardsdatascience.com/proximal-policy-optimization-tutorial-part-1-actor-critic-method-d53f9afffbf6

use bevy::prelude::*;
use tch::{
    nn::{self, OptimizerConfig},
};


use crate::{environment::*};

pub struct PpoMLPlugin;
impl Plugin for PpoMLPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_non_send_resource::<PpoML>()
            .add_system_to_stage( CoreStage::Update, update_action.exclusive_system());
    }
}


pub struct PpoML {
    pub model: nn::Sequential,
    pub opt: nn::Optimizer<nn::Adam>,
    pub steps: Vec<EnvironmentState>,
}

impl FromWorld for PpoML {
    fn from_world(world: &mut World) -> Self {
        let space = world
            .get_resource::<Environment>()
            .expect("Res<EnvSpace> not found.");

        let vs = nn::VarStore::new(tch::Device::Cpu);

        let p = &vs.root();
        let observation_space = space.observation_space as i64;
        let action_space = space.action_space as i64;

        println!("shape: in {}, out {}", observation_space, action_space);

        // What model is this
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

impl PpoML {
    // Train the model via policy gradient on the rollout data.
    fn train(&mut self) {

    }
}

fn update_action(world: &mut World) {

    let world_cell = world.cell();

    // Get env state and run it though our model giving us an action
    let mut ml = world_cell.get_non_send_mut::<PpoML>().unwrap();
    let env_state = world_cell.get_resource_mut::<EnvironmentState>().unwrap();
    let mut _env_action = world_cell.get_resource_mut::<EnvironmentAction>().unwrap();
    let mut env_counter = world_cell.get_resource_mut::<EnvironmentCounter>().unwrap();

    // have we run enough steps this epoch, if so train
    if env_state.is_done && env_counter.step > env_counter.step_max {
        let sum_r: f32 = 0.0;
        println!(
            "epoch: {:<3} episodes: {:<5} avg reward per episode: {:.2}",
            env_counter.epoch,
            env_counter.episode,
            sum_r / env_counter.episode as f32
        );

        ml.train();

        env_counter.episode = 0;
        env_counter.step = 0;
        env_counter.epoch += 1;
    }

    // have we completed this episode
    if env_state.is_done {
        env_counter.episode += 1;
    }
    env_counter.step += 1;
}

