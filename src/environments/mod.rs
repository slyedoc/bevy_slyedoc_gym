pub mod acrobot;
pub mod cartpole;
pub mod mountaincar;
pub mod pendulum;
pub mod flappy;
pub mod breakout;

use std::{fmt, str::FromStr};


#[derive(Clone, Debug)]
pub enum EnvironmentType {
    Acrobot,
    CartPole,
    MountainCar,
    Pendulum,
    Flappy,
    Breakout
}

#[allow(dead_code)]
pub enum EnvironmentMode {
    RenderHuman,
    Render,
    Simulation,
}


impl FromStr for EnvironmentType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "acrobot" => Ok(EnvironmentType::Acrobot),
            "cartpole" => Ok(EnvironmentType::CartPole),
            "mountaincar" => Ok(EnvironmentType::MountainCar),
            "pendulum" => Ok(EnvironmentType::Pendulum),
            "flappy" => Ok(EnvironmentType::Flappy),
            "breakout" => Ok(EnvironmentType::Breakout),
            _ => Err("No environment match!"),
        }
    }
}

impl fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnvironmentType::Acrobot => write!(f, "Acrobot"),
            EnvironmentType::CartPole => write!(f, "Cart Pole"),
            EnvironmentType::MountainCar => write!(f, "Mountain Car"),
            EnvironmentType::Pendulum => write!(f, "Pendulum"),
            EnvironmentType::Flappy => write!(f, "Flappy"),
            EnvironmentType::Breakout => write!(f, "Breakout"),
        }
    }
}
