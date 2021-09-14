use clap::{AppSettings, Clap};

use crate::{environments::EnvironmentType, models::ModelType};

#[derive(Clone, Debug, Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Config {
    #[clap(short, long, default_value = "acrobot", possible_values = &["acrobot", "balldrop", "cartpole", "mountaincar", "pendulum"])]
    pub enviroment: EnvironmentType,

    #[clap(short, long, default_value = "pg", possible_values = &["ddpg", "pg"])]
    pub model: ModelType,

    #[clap(short, long)]
    pub simulation: bool,
}
