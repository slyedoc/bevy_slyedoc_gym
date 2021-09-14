use bevy::{ecs::component::Component, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::environment::*;

#[derive(Debug)]
pub struct AcrobotPlugin {
    pub render: bool,
}

impl Plugin for AcrobotPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2,  4);

        app.add_startup_system(setup_physics.system())
        .add_system(step.system());

        if self.render {
            app.add_system(setup_graphics.system())
                .add_system(keyboard_input.system());
        }
    }
}

fn step(mut env: ResMut<EnvironmentState>) {
    env.reward += 0.1;
    env.is_done = false;
}


fn setup_graphics(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 100.0));
    commands.spawn_bundle(camera);
}

// Makers
struct Link1;
struct Link2;
struct Goal;

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    // Scaling up, see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
    rapier_config.scale = 50.0;

    let link_size = Vec2::new(0.2, 1.0);

    // Create static mount point
    let anchor = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.2),
            //collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::YELLOW))
        .id();

    // Create links(arms)
    let l1 = create_link(
        &mut commands,
        Vec2::new(0.0, -link_size.y),
        link_size,
        Color::GRAY,
        Link1,
    );
    let l2 = create_link(
        &mut commands,
        Vec2::new(0.0, -link_size.y * 3.0),
        link_size,
        Color::GRAY,
        Link2,
    );

    // Add 1st Ball joint
    let joint = BallJoint::new(
        Vec2::ZERO.into(),                  // static anchor
        Vec2::new(0.0, link_size.y).into(), // top of first link
    );
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint, anchor, l1));

    // Add 2nd Ball joint
    let joint2 = BallJoint::new(
        Vec2::new(0.0, -link_size.y).into(), // bottom first link
        Vec2::new(0.0, link_size.y).into(),  // top of second link
    );
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint2, l1, l2));

    // Create the goal line
    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 2.5).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(100.0, 0.05),
            collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::BLACK))
        .insert(Goal);
}

fn create_link(
    commands: &mut Commands,
    pos: Vec2,
    size: Vec2,
    color: Color,
    component: impl Component,
) -> Entity {
    commands
        .spawn_bundle(RigidBodyBundle {
            position: pos.into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(size.x, size.y),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            },
            collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(color))
        .insert(component)
        .id()
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyForces, With<Link2>>,
    keyboard: Res<Input<KeyCode>>,
) {
    for mut rb_forces in rigid_bodies.iter_mut() {
        if keyboard.pressed(KeyCode::A) {
            rb_forces.force = Vec2::new(-10.0, 0.0).into();
        }
        if keyboard.pressed(KeyCode::D) {
            rb_forces.force = Vec2::new(10.0, 0.0).into();
        }
    }
}

/* A system that displays the events. */
fn _display_events(
    mut intersection_events: EventReader<IntersectionEvent>,
    mut contact_events: EventReader<ContactEvent>,
) {
    for intersection_event in intersection_events.iter() {
        println!("Received intersection event: {:?}", intersection_event);
    }

    for contact_event in contact_events.iter() {
        println!("Received contact event: {:?}", contact_event);
    }
}
