use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::environment::*;


pub struct BalldropPlugin {
    pub render: bool,
}

impl Plugin for BalldropPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2,  4);

        app.add_startup_system(setup_physics.system());

        if self.render {
            app.add_system(setup_graphics.system());
        }
    }
}

fn setup_graphics(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 250.0, 100.0));
    commands.spawn_bundle(camera);
}

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    // Scaling up, see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
    rapier_config.scale = 50.0;

    // Create Ground
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(100.0, 0.1),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::BLACK));

    /* Create the bouncing ball. */
    create_ball(&mut commands, 0.1, 1.0, Color::RED);
    create_ball(&mut commands, 0.2, 5.0, Color::BLUE);
    create_ball(&mut commands, 0.3, 10.0, Color::GREEN);
    create_ball(&mut commands, 0.4, 15.0, Color::GOLD);
    create_ball(&mut commands, 0.5, 20.0, Color::PURPLE);
}

fn create_ball(commands: &mut Commands, x: f32, y: f32, color: Color) {
    let rigid_body = RigidBodyBundle {
        position: Vec2::new(x, y).into(),
        ..Default::default()
    };
    let collider = ColliderBundle {
        shape: ColliderShape::ball(0.5),
        material: ColliderMaterial {
            restitution: 0.7,
            ..Default::default()
        },
        ..Default::default()
    };
    commands
        .spawn_bundle(rigid_body)
        .insert_bundle(collider)
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(color));
}
