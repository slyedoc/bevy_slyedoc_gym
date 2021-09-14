use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::environment::*;

#[derive(Debug)]
pub struct PendulumPlugin {
    pub render: bool,
}

impl Plugin for PendulumPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);
        
        app.add_startup_system(setup_physics.system());

        if self.render {
            app.add_system(setup_graphics.system())
                .add_system(keyboard_input.system());
        }
    }
}

fn setup_graphics(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 100.0));
    commands.spawn_bundle(camera);
}

// Makers
struct Link;

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.scale = 50.0;

    let link_size = Vec2::new(0.4, 2.0);

    let anchor = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(link_size.x * 0.8),
            collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::YELLOW))
        .id();

    let link = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, -link_size.y).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(link_size.x, link_size.y),
            //collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::GRAY))
        .insert(Link)
        .id();

    let joint = BallJoint::new(Vec2::ZERO.into(), Vec2::new(0.0, link_size.y).into());
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint, anchor, link));
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyForces, With<Link>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut rb_forces in rigid_bodies.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            rb_forces.force = Vec2::new(-10.0, 0.0).into();
        }
        if keyboard_input.pressed(KeyCode::D) {
            rb_forces.force = Vec2::new(10.0, 0.0).into();
        }
    }
}
