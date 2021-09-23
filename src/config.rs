use clap::{AppSettings, Clap};

use crate::{environments::EnvironmentType, models::ModelType};

#[derive(Clone, Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Config {
    #[clap(short, long, default_value = "flappy", possible_values = &["acrobot", "cartpole", "mountaincar", "pendulum", "flappy", "breakout"])]
    pub environment: EnvironmentType,

    #[clap(short, long, default_value = "pg", possible_values = &["ppo", "pg", "neat"])]
    pub model: ModelType,

    #[clap(short, long)]
    pub human: bool,

    #[clap(short, long)]
    pub simulation: bool,
}
