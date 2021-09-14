use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::environment::*;

#[derive(Debug)]
pub struct PendulumPlugin {
    pub render: bool,
}

impl Plugin for PendulumPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 2);

        app.add_startup_system(setup_physics.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system());

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

const ACTION_FORCE: f32 = 3000.0; // F / dt
fn take_action(
    mut env_action: ResMut<EnvironmentAction>,
    mut cart: Query<&mut RigidBodyForces, With<Link>>,
    params: Res<IntegrationParameters>,
) {
    //println!("take_action: {}", env_action.take);
    if env_action.take {
        for mut rb_f in cart.iter_mut() {
            match env_action.action {
                0 => rb_f.force = Vec2::new(-ACTION_FORCE * params.dt, 0.0).into(),
                1 => rb_f.force = Vec2::new(ACTION_FORCE * params.dt, 0.0).into(),
                _ => panic!("action invalid: {}", env_action.action),
            }
        }
        env_action.take = false;
    }
}

// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    env_action: Res<EnvironmentAction>,
    link: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Link>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
) {
    //println!("udpate_state");
    // Find our obserables
    let mut cart_pos_x = 0.0;
    let mut cart_vel = 0.0;
    for (rb_pos, rb_vel) in link.iter() {
        cart_pos_x = rb_pos.position.translation.x;
        cart_vel = rb_vel.linvel[0];
    }

    // Update state using that info
    state.observation = vec![cart_pos_x, cart_vel];
    state.action = env_action.action;
    state.reward = 1.0;

    state.is_done = false; // TODO: This corrently only works on cartpole
    if state.is_done {
        ev_reset.send(EnvironmentResetEvent);
    }
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
