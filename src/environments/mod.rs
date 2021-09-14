pub mod acrobot;
pub mod balldrop;
pub mod cartpole;
pub mod mountaincar;
pub mod pendulum;

use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub enum EnvironmentType {
    Acrobot,
    Balldrop,
    CartPole,
    MountainCar,
    Pendulum,
}

impl FromStr for EnvironmentType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "acrobot" => Ok(EnvironmentType::Acrobot),
            "balldrop" => Ok(EnvironmentType::Balldrop),
            "cartpole" => Ok(EnvironmentType::CartPole),
            "mountaincar" => Ok(EnvironmentType::MountainCar),
            "pendulum" => Ok(EnvironmentType::Pendulum),
            _ => Err("no enviroment match"),
        }
    }
}

impl fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnvironmentType::Acrobot => write!(f, "Acrobot"),
            EnvironmentType::Balldrop => write!(f, "Balldrop"),
            EnvironmentType::CartPole => write!(f, "Cart Pole"),
            EnvironmentType::MountainCar => write!(f, "Mountain Car"),
            EnvironmentType::Pendulum => write!(f, "Pendulum"),
        }
    }
}
