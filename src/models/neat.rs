// Neat example.
//
// https://github.com/suhdonghwi/neat

use bevy::prelude::*;
//use neat::network::Network;
//use neat::{innovation_record::InnovationRecord, network::feedforward::Feedforward, pool::Pool};

use crate::environment::*;

use super::MLModel;

pub struct NeatMLPlugin;
impl Plugin for NeatMLPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_non_send_resource::<NeatML>()
            .add_system_to_stage(CoreStage::Update, NeatML::update_action.exclusive_system());
    }
}

impl FromWorld for NeatML {
    fn from_world(world: &mut World) -> Self {
        let space = world
            .get_resource::<Environment>()
            .expect("Res<EnvSpace> not found.");
        let observation_space = space.observation_space as i64;
        let action_space = space.action_space as i64;
        println!("shape: in {}, out {}", observation_space, action_space);

        Self {}
    }
}

pub struct NeatML {
    //pub model: nn::Sequential,
//pub opt: nn::Optimizer<nn::Adam>,
//pub steps: Vec<EnvironmentState>,
}

impl MLModel for NeatML {
    fn update_action(world: &mut World) {
        let world_cell = world.cell();

        // Get env state and run it though our model giving us an action
        let mut _ml = world_cell.get_non_send_mut::<NeatML>().unwrap();
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
            // Do what ever you need to do to train

            env_counter.episode = 0;
            env_counter.step = 0;
            env_counter.epoch += 1;
        } else if env_state.is_done {
            env_counter.episode += 1;
        }
        env_counter.step += 1;
    }
}
