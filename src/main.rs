mod config;
mod environment;
mod environments;
mod helpers;
mod models;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_rapier2d::physics::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_rapier2d::prelude::IntegrationParameters;
use bevy_rapier2d::render::RapierRenderPlugin;
use clap::Clap;
use config::Config;
use environments::*;

use crate::environment::{EnvironmentCounter, EnvironmentResetEvent};
use crate::environments::acrobot::AcrobotPlugin;
use crate::environments::cartpole::CartPolePlugin;
use crate::environments::flappy::FlappyPlugin;
use crate::environments::mountaincar::MountainCarPlugin;
use crate::environments::pendulum::PendulumPlugin;
use crate::models::neat::NeatMLPlugin;
use crate::models::policy_gradient::PolicyGradientModelPlugin;
use crate::models::ppo::PpoMLPlugin;
use crate::models::ModelType;

fn main() {

    let config = Config::parse();
    let mut app = App::build();
    println!("{:?}", config);

    // Setup bevy
    if config.simulation {
        app.add_plugins(MinimalPlugins);
    } else {
        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(ClearColor(Color::WHITE))
            .insert_resource(WindowDescriptor {
                title: config.environment.to_string(),
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_system(keyboard_input.system());
    }

    // Setup Common Resources
    app.insert_resource(EnvironmentCounter::default())
        .add_event::<EnvironmentResetEvent>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup_physics.system());

    if !config.simulation {
        app.add_plugin(RapierRenderPlugin);
    }

    // Add enviroment
    match config.environment {
        EnvironmentType::Acrobot => app.add_plugin(AcrobotPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::CartPole => app.add_plugin(CartPolePlugin {
            render: !config.simulation,
        }),
        EnvironmentType::MountainCar => app.add_plugin(MountainCarPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::Pendulum => app.add_plugin(PendulumPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::Flappy => app.add_plugin(FlappyPlugin {
            render: !config.simulation,
        }),
    };


    if !config.human {

        // If no human
        match config.model {
            ModelType::PolicyGradient => app.add_plugin(PolicyGradientModelPlugin),
            ModelType::PPO => app.add_plugin(PpoMLPlugin),
            ModelType::Neat => app.add_plugin(NeatMLPlugin),
        };
    }

    app.run();
}

fn setup_physics(
    mut params: ResMut<IntegrationParameters>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_config.timestep_mode = TimestepMode::VariableTimestep;
    params.dt = 1.0 / 60.0
}

fn keyboard_input(mut exit: EventWriter<AppExit>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}
