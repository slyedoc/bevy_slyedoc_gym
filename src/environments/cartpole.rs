use crate::{environment::*, models::policy_gradient::PolicyGradientModel};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;
pub struct CartPolePlugin {
    pub human: bool,
    pub render: bool,
}

impl Plugin for CartPolePlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);

        if !self.human {
            app.init_non_send_resource::<PolicyGradientModel>()
                .add_system_to_stage(CoreStage::Update, update_action.exclusive_system());
        }

        app.add_startup_system(setup_environment.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system())
            .add_system(reset_listener.system());

        if self.render {
            app.add_system_to_stage(CoreStage::Update, keyboard_input.system());
        }
    }
}

fn keyboard_input(mut env_state: ResMut<EnvironmentState>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::A) {
        env_state.action = Some(0);
    }

    if keyboard_input.pressed(KeyCode::D) {
        env_state.action = Some(1);
    }
}
// Makers to identify entities
struct Cart;
struct Pole;
struct Ground;

const RAPIER_SCALE: f32 = 50.0; // Very useful to zoom in and out to see whats going on
                                // Also see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
const CART_RANGE: f32 = 4.8;

const POLE_ANGLE_LIMIT: f32 = 0.418; // 24 degrees
const POLE_INIT_FORCE_LIMIT: f32 = 0.5;
const ACTION_FORCE: f32 = 5000.0;

fn take_action(
    mut env_state: ResMut<EnvironmentState>,
    mut cart: Query<&mut RigidBodyForces, With<Cart>>,
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
        // Clear after use for now
        env_state.action = None;
    }
}

fn update_action(world: &mut World) {
    let world_cell = world.cell();

    let mut pg = world_cell
        .get_non_send_mut::<PolicyGradientModel>()
        .unwrap();

    // Get env state and run it though our model giving us an action
    let mut env_state = world_cell.get_resource_mut::<EnvironmentState>().unwrap();

    // Policy Gradient
    env_state.action = Some(pg.step(&env_state.observation, env_state.reward, env_state.is_done));
}

// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    pole: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Pole>>,
    cart: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Cart>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
    mut counter: Local<usize>,
) {
    // Find our observables
    let mut cart_pos_x = 0.0;
    let mut cart_vel = 0.0;
    let mut pole_angle = 0.0;
    let mut pole_angle_vel = 0.0;
    for (rb_pos, rb_vel) in cart.iter() {
        cart_pos_x = rb_pos.position.translation.x;
        cart_vel = rb_vel.linvel[0];
    }
    for (rb_pos, rb_vel) in pole.iter() {
        pole_angle = rb_pos.position.rotation.angle();
        pole_angle_vel = rb_vel.angvel;
    }

    // Update state using that info
    state.observation = vec![cart_pos_x, cart_vel, pole_angle, pole_angle_vel];
    state.reward = *counter as f32;
    let done = reset_check(cart_pos_x, pole_angle);
    state.is_done = Some(done);
    if done {
        ev_reset.send(EnvironmentResetEvent);
        *counter = 0;
    } else {
        *counter += 1;
    }
}



fn reset_check(cart_pos_x: f32, pole_angle: f32) -> bool {
    let mut reset = false;
    if cart_pos_x.abs() > CART_RANGE {
        reset = true;
    }
    if pole_angle.abs() > POLE_ANGLE_LIMIT {
        reset = true;
    }
    reset
}

fn setup_environment(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    config: Res<EnvironmentConfig>,
) {
    rapier_config.scale = RAPIER_SCALE;

    if config.render {
        let mut camera = OrthographicCameraBundle::new_2d();
        camera.transform = Transform::from_translation(Vec3::new(0.0, 150.0, 50.0));
        commands.spawn_bundle(camera);
    }

    // Create the ground, will serve as anchor point for PrismaticJoint with cart
    let ground = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(100.0, 0.1),
            //collider_type: ColliderType::Sensor,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::BLACK))
        .insert(Ground)
        .id();

    spawn_cart_pole(commands, ground);
}

// Creates the cart and pole
// Assumes empty Environment except ground
fn spawn_cart_pole(mut commands: Commands, ground: Entity) {
    let pole_size = Vec2::new(0.2, 4.0);
    let cart_size = Vec2::new(4.0, 2.0);

    let cart = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Sensor,
            shape: ColliderShape::cuboid(cart_size.x * 0.5, cart_size.y * 0.5),
            mass_properties: ColliderMassProps::Density(2.0),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::MAROON))
        .insert(Cart)
        .id();

        let mut rnd = rand::thread_rng();

    let pole = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, (pole_size.y * 0.5) + (cart_size.y * 0.5)).into(),
            // adding random velocity so its not stable
            velocity: RigidBodyVelocity {
                linvel: Vec2::new(rnd.gen_range(-POLE_INIT_FORCE_LIMIT..POLE_INIT_FORCE_LIMIT), 0.0).into(),
                angvel: 0.0
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(pole_size.x * 0.5, pole_size.y * 0.5),
            collider_type: ColliderType::Sensor,
            mass_properties: ColliderMassProps::Density(0.5),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::SILVER))
        .insert(Pole)
        .id();
    let x = Vector::x_axis();
    let mut cart_rollers_joint =
        PrismaticJoint::new(Vec2::new(-50.0, 0.0).into(), x, Vec2::ZERO.into(), x);
    cart_rollers_joint.limits = [0.0, 100.0];
    commands
        .spawn()
        .insert(JointBuilderComponent::new(cart_rollers_joint, ground, cart));
    let pole_joint = BallJoint::new(
        Vec2::new(0.0, cart_size.y * 0.5).into(),
        Vec2::new(0.0, -pole_size.y * 0.5).into(),
    );
    commands
        .spawn()
        .insert(JointBuilderComponent::new(pole_joint, cart, pole));

}

fn reset_listener(
    mut commands: Commands,
    mut ev_reset: EventReader<EnvironmentResetEvent>,
    query_set: QuerySet<(
        Query<Entity, With<Cart>>,
        Query<Entity, With<Pole>>,
        Query<Entity, With<Ground>>,
    )>,
) {
    let mut reset = false;
    for _ in ev_reset.iter() {
        reset = true;
    }
    if reset {
        for e in query_set.q0().iter() {
            commands.entity(e).despawn_recursive();
        }
        for e in query_set.q1().iter() {
            commands.entity(e).despawn_recursive();
        }

        // TODO: Do joints despawn with there entity?, if not need to do it here

        let g = query_set.q2().iter().next();
        if let Some(g) = g {
            spawn_cart_pole(commands, g);
        }
    }
}


