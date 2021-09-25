use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::{BreakoutConfig, Player};

pub fn player_movement_human(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<&mut RigidBodyPosition, With<Player>>,
    params: Res<IntegrationParameters>,
    config: Res<BreakoutConfig>,
) {
    let movement = config.player_speed * params.dt;
    let limit = config.board_size_half.x - config.player_size_half.x - config.board_line_size_half;
    for mut rb_pos in players.iter_mut() {
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        let x : f32 = if left {
            rb_pos.position.translation.x - movement
        } else if right {
            rb_pos.position.translation.x + movement
        } else {
            0.0
        };
        if x != 0.0 {
            rb_pos.next_position.translation.x = x.clamp(-limit, limit);
        }
    }
}
