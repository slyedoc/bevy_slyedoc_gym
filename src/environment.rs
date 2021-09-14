#[derive(Debug)]
pub struct Environment {
    pub action_space: usize,
    pub observation_space: usize,
}

#[derive(Debug, Clone)]
pub struct EnvironmentState {
    pub observation: Vec<f32>,
    pub reward: f32,
    pub is_done: bool,
}

#[derive(Debug)]
pub struct EnvironmentAction {
    pub action: usize,
}

#[derive(Debug)]
pub struct EnvironmentCounter {
    pub epoch: usize, //
    pub epoch_max: usize,
    pub step: usize,
    pub step_max: usize,
}

impl Default for EnvironmentCounter {
    fn default() -> Self {
        Self {
            epoch: 0,
            epoch_max: 50,
            step: 0,
            step_max: 5000,
        }
    }
}

pub struct EnvironmentResetEvent;

// Helper fuction for adding the 2 resources uses to by models
pub fn insert_env_resources(
    app: &mut bevy::prelude::AppBuilder,
    action_space: usize,
    observation_space: usize,
) {
    app.insert_resource(Environment {
        action_space,
        observation_space,
    })
    .insert_resource(EnvironmentState {
        observation: vec![0.0; observation_space],
        reward: 0.0,
        is_done: false,
    })
    .insert_resource(EnvironmentAction {
        action: action_space,
    });
}
