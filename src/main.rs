mod config;
mod environment;
mod environments;
mod helpers;
mod models;
mod menu;

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

use crate::menu::MenuPlugin;



#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Loading, // Asset Loading
    Menu,    // Main Menu
    Environment(EnvironmentType),
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
                title: match config.environment {
                    Some(e) => e.to_string(),
                    None => "Bevy Slyedoc Gym".to_string(),
                },
                ..Default::default()
            })
            .insert_resource(WorldInspectorParams {
                enabled: true,
                despawnable_entities: false,
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(EguiPlugin);
    }

    // Setup Common Resources
    app
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(DebugLinesPlugin)
    .insert_resource(DebugLines {
        ..Default::default()
    })
    .add_startup_system(setup_physics.system());

    if !config.simulation {
        app.add_plugin(RapierRenderPlugin);
        app.add_plugin(MenuPlugin);
    }

    // Add environment
    match config.environment {
        Some(e) => {
            load_environment(&mut app, e, !config.simulation, config.human);
            app.add_state(AppState::Environment(e));
        }
        None => {
            app.add_state(AppState::Menu);
        },
    }

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



fn enable_physics_profiling(mut pipeline: ResMut<PhysicsPipeline>) {
    pipeline.counters.enable()
}
