use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::environment::*;

#[derive(Debug)]
pub struct PendulumPlugin {
    pub render: bool,

}

enum PendulumState {
    Playing,
    Reset,
}

impl Plugin for PendulumPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 2);

        app.add_system_set( SystemSet::on_in_stack_update(s))
            .add_startup_system(setup_environment.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system());

        if self.render {
            app.add_system_to_stage(CoreStage::Update, keyboard_input.system());
        }
    }
}

const RAPIER_SCALE: f32 = 50.0;
const ACTION_FORCE: f32 = 3000.0;
const LINK_SIZE_HALF_X: f32 = 1.0;
const LINK_SIZE_HALF_Y: f32 = 10.0;

fn keyboard_input(keyboard_input: Res<Input<KeyCode>>, mut env_state: ResMut<EnvironmentState>) {
    if keyboard_input.pressed(KeyCode::A) {
        env_state.action = Some(0);
    }
    if keyboard_input.pressed(KeyCode::D) {
        env_state.action = Some(1);
    }
}

// Makers
struct Link;

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

    let anchor = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(LINK_SIZE_HALF_X),
            collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::YELLOW))
        .id();

    let link = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, -LINK_SIZE_HALF_X).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(LINK_SIZE_HALF_X, LINK_SIZE_HALF_Y),
            //collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::GRAY))
        .insert(Link)
        .id();

    let joint = BallJoint::new(Vec2::ZERO.into(), Vec2::new(0.0, LINK_SIZE_HALF_Y).into());
    commands
        .spawn()
        .insert(JointBuilderComponent::new(joint, anchor, link));
}


fn take_action(
    env_state: Res<EnvironmentState>,
    mut cart: Query<&mut RigidBodyForces, With<Link>>,
    params: Res<IntegrationParameters>,
) {
    if let Some(action) = env_state.action {
        for mut rb_f in cart.iter_mut() {
            match action {
                0 => rb_f.force = Vec2::new(-ACTION_FORCE * params.dt, 0.0).into(),
                1 => rb_f.force = Vec2::new(ACTION_FORCE * params.dt, 0.0).into(),
                _ => panic!("action invalid: {}", action),
            }
        }
    }
}

// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    link: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Link>>,
) {
    // Find our observables
    let mut cart_pos_x = 0.0;
    let mut cart_vel = 0.0;
    for (rb_pos, rb_vel) in link.iter() {
        cart_pos_x = rb_pos.position.translation.x;
        cart_vel = rb_vel.linvel[0];
    }

    // Update state using that info
    state.observation = vec![cart_pos_x, cart_vel];
    state.reward = 1.0;
    state.is_done = None;
}


