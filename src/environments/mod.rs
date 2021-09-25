pub mod acrobot;
pub mod cartpole;
pub mod mountaincar;
pub mod pendulum;
pub mod flappy;
pub mod breakout;

use std::{fmt, str::FromStr};

use bevy::prelude::AppBuilder;

use self::{acrobot::AcrobotPlugin, breakout::BreakoutPlugin, cartpole::CartPolePlugin, flappy::{FlappyConfig, FlappyPlugin}, mountaincar::MountainCarPlugin, pendulum::PendulumPlugin};


#[derive(Copy,Clone,Debug,Eq, PartialEq, Hash)]
pub enum EnvironmentType {
    Acrobot,
    CartPole,
    MountainCar,
    Pendulum,
    Flappy,
    Breakout
}

#[allow(dead_code)]
#[derive(Debug)]
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

pub fn load_environment(app: &mut AppBuilder, env: EnvironmentType, render: bool, human: bool) {
    match env {
        EnvironmentType::Acrobot => app.add_plugin(AcrobotPlugin { render: render }),
        EnvironmentType::CartPole => app.add_plugin(CartPolePlugin {
            human: human,
            render: render,
        }),
        EnvironmentType::MountainCar => app.add_plugin(MountainCarPlugin { render: render }),
        EnvironmentType::Pendulum => app.add_plugin(PendulumPlugin { render: render }),
        EnvironmentType::Flappy => app.add_plugin(FlappyPlugin {
            config: FlappyConfig {
                render: render,
                human: human,
            },
        }),
        EnvironmentType::Breakout => app.add_plugin(BreakoutPlugin {
            render: render,
            human: human,
        }),
    };
}