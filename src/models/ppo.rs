// TODO: PPO

// https://towardsdatascience.com/proximal-policy-optimization-tutorial-part-1-actor-critic-method-d53f9afffbf6

use bevy::prelude::*;
use crate::environment::*;

pub struct PpoMLPlugin;
impl Plugin for PpoMLPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_non_send_resource::<PpoML>()
            .add_system_to_stage(CoreStage::Update, update_action.exclusive_system());
    }
}

fn update_action(world: &mut World) {
    let world_cell = world.cell();

    let mut m = world_cell
        .get_non_send_mut::<PpoML>()
        .unwrap();

    // Get env state and run it though our model giving us an action
    let mut env_state = world_cell.get_resource_mut::<EnvironmentState>().unwrap();

    // TODO: use our PPO ML
    let action = m.step(&env_state.observation, env_state.reward, env_state.is_done);
    env_state.action = Some(action);
}

pub struct PpoML {

}

impl FromWorld for PpoML {
    fn from_world(world: &mut World) -> Self {
        let space = world
            .get_resource::<Environment>()
            .expect("Res<EnvSpace> not found.");

        let observation_space = space.observation_space as i64;
        let action_space = space.action_space as i64;

        println!("shape: in {}, out {}", observation_space, action_space);

        Self {

        }
    }
}

impl PpoML {
    // Train the model via policy gradient on the rollout data.
    fn step(&mut self, _observations: &[f32], _reward: f32, _done: Option<bool>) -> usize {
        0
    }
}

