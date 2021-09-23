use bevy::{ecs::component::Component, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::environment::*;

#[derive(Debug)]
pub struct AcrobotPlugin {
    pub render: bool,
}

impl Plugin for AcrobotPlugin {
    fn build(&self, app: &mut AppBuilder) {

        app.add_startup_system(setup_environment.system());

        if self.render {
            app.add_system_to_stage(CoreStage::Update, update_human.system());
            println!("Keys: A and D");
        }

        println!("WARNING: No models really support this.");
    }
}

const RAPIER_SCALE: f32 = 50.0;
const LINK_SIZE_HALF_X: f32 = 0.2;
const LINK_SIZE_HALF_Y: f32 = 1.0;
const ACTION_FORCE: f32 = 1000.0;

fn update_human(
    keyboard: Res<Input<KeyCode>>,
    mut link: Query<&mut RigidBodyForces, With<Link1>>,
    params: Res<IntegrationParameters>,
) {
    for mut rb_f in link.iter_mut() {
        if keyboard.pressed(KeyCode::A) {
            rb_f.force = Vec2::new(-ACTION_FORCE * params.dt, 0.0).into();
        }
        if keyboard.pressed(KeyCode::D) {
            rb_f.force = Vec2::new(ACTION_FORCE * params.dt, 0.0).into();
        }
    }
}

#[allow(dead_code)]
fn update_pg(
    link1: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Link1>>,
    link2: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Link2>>,
) {
    // Find our observables
    let mut _link1_pos_x = 0.0;
    let mut _link1_vel = 0.0;
    let mut _link2_pos_x = 0.0;
    let mut _link2_vel = 0.0;

    for (rb_pos, rb_vel) in link1.iter() {
        _link1_pos_x = rb_pos.position.translation.x;
        _link1_vel = rb_vel.linvel[0];
    }

    for (rb_pos, rb_vel) in link2.iter() {
        _link2_pos_x = rb_pos.position.translation.x;
        _link2_vel = rb_vel.linvel[0];
    }

    // TODO: Update state using that info
}
// Makers
struct Link1;
struct Link2;
struct Goal;

fn setup_environment(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    config: Res<EnvironmentConfig>,
) {
    rapier_config.scale = RAPIER_SCALE;

    if config.render {
        let mut camera = OrthographicCameraBundle::new_2d();
        camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 100.0));
        commands.spawn_bundle(camera);
    }

    // Create static mount point
    let anchor = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(LINK_SIZE_HALF_X),
            //collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::YELLOW))
        .id();

    // Create links(arms)
    let link1 = create_link(
        &mut commands,
        Vec2::new(0.0, -LINK_SIZE_HALF_Y),
        Color::GRAY,
        Link1,
    );

    if config.render {
        // Add Joint Visualization - cosmetic only
        commands
            .spawn_bundle(ColliderBundle {
                shape: ColliderShape::ball(LINK_SIZE_HALF_X),
                collider_type: ColliderType::Sensor,
                ..Default::default()
            })
            .insert(ColliderParent {
                handle: link1.handle(),
                pos_wrt_parent: Vec2::new(0.0, -LINK_SIZE_HALF_Y).into(),
            })
            .insert(ColliderPositionSync::Discrete)
            .insert(ColliderDebugRender::from(Color::BLACK));
    }
    let l2 = create_link(
        &mut commands,
        Vec2::new(0.0, -LINK_SIZE_HALF_Y * 3.0),
        Color::GRAY,
        Link2,
    );

    // Add 1st Ball joint
    let joint = BallJoint::new(
        Vec2::ZERO.into(),                       // static anchor
        Vec2::new(0.0, LINK_SIZE_HALF_Y).into(), // top of first link
    );
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint, anchor, link1));

    // Add 2nd Ball joint
    let joint2 = BallJoint::new(
        Vec2::new(0.0, -LINK_SIZE_HALF_Y).into(), // bottom first link
        Vec2::new(0.0, LINK_SIZE_HALF_Y).into(),  // top of second link
    );
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint2, link1, l2));

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
    color: Color,
    component: impl Component,
) -> Entity {
    commands
        .spawn_bundle(RigidBodyBundle {
            position: pos.into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(LINK_SIZE_HALF_X, LINK_SIZE_HALF_Y),
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
