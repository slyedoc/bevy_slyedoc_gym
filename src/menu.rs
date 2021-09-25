use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui::*, *};

use crate::{AppState, environments::EnvironmentType};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_update(AppState::Menu)
        .with_system(draw_menu.system())
        .with_system(keyboard_input.system())
    );
    }
}

fn draw_menu(
    egui_ctx: Res<EguiContext>,
    mut exit: EventWriter<AppExit>,
    mut state: ResMut<State<AppState>>,
) {
    SidePanel::left("menu")
        .default_width(200.0)
        .resizable(false)
        .show(egui_ctx.ctx(), |ui| {
            ui.heading("Bevy Slyedoc Gym");
            ui.separator();
            ui.label("Games");
            if ui.button( EnvironmentType::Flappy.to_string()).clicked() {
                state.set( AppState::Environment(EnvironmentType::Flappy)).unwrap();
            }
            if ui.button( EnvironmentType::Breakout.to_string()).clicked() {
                state.set( AppState::Environment(EnvironmentType::Breakout)).unwrap();
            }
            ui.separator();
            if ui.button("Settings").clicked() {}

            ui.separator();
            if ui.button("Exit").clicked() {
                exit.send(AppExit);
            }
        });
}

fn keyboard_input(mut exit: EventWriter<AppExit>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}