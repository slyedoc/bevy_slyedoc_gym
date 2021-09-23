mod config;
mod environment;
mod environments;
mod helpers;
mod models;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorParams;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_rapier2d::physics::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_rapier2d::prelude::{IntegrationParameters, PhysicsPipeline};
use bevy_rapier2d::render::RapierRenderPlugin;
use clap::Clap;
use config::Config;
use environments::*;

use crate::environment::{EnvironmentConfig, EnvironmentResetEvent};
use crate::environments::acrobot::AcrobotPlugin;
use crate::environments::breakout::{BreakoutConfig, BreakoutPlugin};
use crate::environments::cartpole::CartPolePlugin;
use crate::environments::flappy::{FlappyConfig, FlappyPlugin};
use crate::environments::mountaincar::MountainCarPlugin;
use crate::environments::pendulum::PendulumPlugin;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MainState {
    Loading,
    Playing,
    Resetting,
}

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
            .insert_resource(WorldInspectorParams {
                enabled: false,
                despawnable_entities: false,
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(EguiPlugin)
            .add_system(keyboard_input.system());
    }

    // Setup Common Resources
    app.insert_resource(EnvironmentConfig {
        render: !config.simulation,
    })
    .add_event::<EnvironmentResetEvent>()
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(DebugLinesPlugin)
    .insert_resource(DebugLines {
        ..Default::default()
    })
    .add_startup_system(setup_physics.system());

    if !config.simulation {
        app.add_plugin(RapierRenderPlugin);
    }

    // Add environment
    match config.environment {
        EnvironmentType::Acrobot => app.add_plugin(AcrobotPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::CartPole => app.add_plugin(CartPolePlugin {
            human: config.human,
            render: !config.simulation,
        }),
        EnvironmentType::MountainCar => app.add_plugin(MountainCarPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::Pendulum => app.add_plugin(PendulumPlugin {
            render: !config.simulation,
        }),
        EnvironmentType::Flappy => app.add_plugin(FlappyPlugin {
            config: FlappyConfig {
                render: !config.simulation,
                human: config.human,
            },
        }),
        EnvironmentType::Breakout => app.add_plugin(BreakoutPlugin {
            config: BreakoutConfig {
                render: !config.simulation,
                human: config.human,
            },
        }),
    };

    app.add_startup_system(enable_physics_profiling.system())
        .run();
}

fn setup_physics(
    mut params: ResMut<IntegrationParameters>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_config.timestep_mode = TimestepMode::VariableTimestep;
    params.dt = 3.0 / 60.0
}

fn keyboard_input(mut exit: EventWriter<AppExit>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn enable_physics_profiling(mut pipeline: ResMut<PhysicsPipeline>) {
    pipeline.counters.enable()
}
