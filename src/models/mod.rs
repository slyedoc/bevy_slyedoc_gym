pub mod ppo;
pub mod policy_gradient;
pub mod neat;

use std::str::FromStr;

use bevy::prelude::World;

#[derive(Clone, Debug)]
pub enum ModelType {
    PPO,
    PolicyGradient,
    Neat,
}

impl FromStr for ModelType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ppo" => Ok(ModelType::PPO),
            "policy_gradient" => Ok(ModelType::PolicyGradient),
            "pg" => Ok(ModelType::PolicyGradient),
            "neat" => Ok(ModelType::Neat),
            _ => Err("no model match"),
        }
    }
}

pub trait MLModel {
    fn update_action(world: &mut World);
}