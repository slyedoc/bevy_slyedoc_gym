use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::models::neat::NeatML;

use super::{BreakoutConfig, Player};

pub fn player_movement_neat(
    mut neat: ResMut<NeatML>,
    mut players: Query<(&Player, &RigidBodyPosition, &mut RigidBodyVelocity)>,
    balls: Query<&RigidBodyPosition, With<Ball>>,
    params: Res<IntegrationParameters>,
    config: Res<BreakoutConfig>
) {
    let ball_pos = balls.single().expect("One ball should exist!");

    for (player, pos, mut rb_vel) in players.iter_mut() {
        let observations = &[
            pos.position.translation.x as f64,      // player x
            pos.position.translation.y as f64,      // player y
            ball_pos.position.translation.x as f64, // ball x
            ball_pos.position.translation.y as f64, // ball y
        ];
        let output = neat.pool.activate_nth(player.index, observations).unwrap();

        // Go left
        rb_vel.linvel = match output[0] {
            o if o < 0.33 => Vec2::new(-config.player_speed * params.dt, 0.0).into(),
            o if o > 0.66 => Vec2::new(config.player_speed * params.dt, 0.0).into(),
            _ => Vec2::ZERO.into(),
        }
    }
}
