mod config;
mod environment;
mod environments;
mod models;
mod helpers;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_rapier2d::physics::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierRenderPlugin;
use clap::Clap;
use config::Config;
use environments::*;

use crate::environment::{EnvironmentResetEvent, EnvironmentCounter};
use crate::environments::acrobot::AcrobotPlugin;
use crate::environments::balldrop::BalldropPlugin;
use crate::environments::cartpole::CartPolePlugin;
use crate::environments::mountaincar::MountainCarPlugin;
use crate::environments::pendulum::PendulumPlugin;
use crate::models::ModelType;
use crate::models::policy_gradient::PolicyGradientModelPlugin;

fn main() {
    let config = Config::parse();
    let mut app = App::build();

    println!("{:?}", config);

    // Setup bevy
    if config.simulation {
        app.add_plugins(MinimalPlugins);
    }
    else {
        app
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(ClearColor(Color::WHITE))
            .insert_resource(WindowDescriptor {
                title: config.enviroment.to_string(),
                ..Default::default()
            })
            .add_plugins(DefaultPlugins);
    }

    // Setup Common Resources
    app.insert_resource(EnvironmentCounter {
        epoch: 0,
        epoch_max: 10,
        step: 0,
        step_max: 100,
    })
    .add_event::<EnvironmentResetEvent>()
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
    if !config.simulation {
        app.add_plugin(RapierRenderPlugin);
    }

    // Add enviroment
    match config.enviroment {
        EnvironmentType::Acrobot => app.add_plugin(AcrobotPlugin { render: !config.simulation }),
        EnvironmentType::Balldrop => app.add_plugin(BalldropPlugin { render: !config.simulation }),
        EnvironmentType::CartPole => app.add_plugin(CartPolePlugin { render: !config.simulation }),
        EnvironmentType::MountainCar => app.add_plugin(MountainCarPlugin { render: !config.simulation }),
        EnvironmentType::Pendulum => app.add_plugin(PendulumPlugin { render: !config.simulation }),
    };

    if !config.simulation {
        app.add_system(keyboard_input.system());
    }

    match config.model {
        ModelType::Ddpg => todo!(),
        ModelType::PolicyGradient => app.add_plugin(PolicyGradientModelPlugin),
    };

    app.run();

    // env.step();
    // match config.render {
    //     true => env,
    //     false => {
    //         for _ in 0..10 {
    //             //env.reset().expect("Unable to reset");
    //             for _ in 0..100 {
    //                 let action: usize = 0;
    //                 let step = env.step(action);
    //                 if step.is_done.is {
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }

    // env.close();
}

fn keyboard_input(mut exit: EventWriter<AppExit>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

pub fn _step_runner(mut app: App) {
    println!("Type stuff into the console");
    app.update();
}

