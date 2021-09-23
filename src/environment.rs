#[derive(Debug)]
pub struct Environment {
    pub action_space: usize,
    pub observation_space: usize,
}

#[derive(Debug, Clone)]
pub struct EnvironmentState {
    pub action: Option<usize>,
    pub observation: Vec<f32>,
    pub reward: f32,
    pub is_done: Option<bool>,
}



pub struct EnvironmentResetEvent;

// Helper function for adding the 2 resources uses to by the gym
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
        is_done: None,
        action: None,
    });
}

pub struct EnvironmentConfig {
    pub render: bool,
}

