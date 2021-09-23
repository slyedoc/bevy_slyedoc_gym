use crate::{environment::*, models::policy_gradient::PolicyGradientModel};
use bevy::{prelude::*, render::camera::Camera};
use bevy_rapier2d::prelude::*;
use rand::Rng;
use tch::{Kind, Tensor};

pub struct CartPolePlugin {
    pub human: bool,
    pub render: bool,
}

impl Plugin for CartPolePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(CartpoleState::Loading)
            .add_system_set(
                SystemSet::on_enter(CartpoleState::Loading).with_system(setup_environment.system()),
            )
            .add_system_set(
                SystemSet::on_enter(CartpoleState::Resetting)
                    .with_system(clean_environment.system()),
            );

        if self.human && self.render {
            app.add_system_set(
                SystemSet::on_update(CartpoleState::Playing).with_system(update_human.system()),
            );
        } else {
            app.insert_non_send_resource(PolicyGradientModel::new(4, 1))
                .add_system_set(
                    SystemSet::on_update(CartpoleState::Playing)
                        .with_system(update_pg.exclusive_system())
                        .with_system(check_pg.exclusive_system()),
                );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CartpoleState {
    Loading,
    Playing,
    Resetting,
}

// Makers to identify entities
struct Cart;
struct Pole;
struct CartPoleClean;

const RAPIER_SCALE: f32 = 50.0; // Very useful to zoom in and out to see whats going on
                                // Also see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
const CART_RANGE: f32 = 4.8;

const POLE_ANGLE_LIMIT: f32 = 0.418; // 24 degrees
const POLE_INIT_FORCE_LIMIT: f32 = 0.5;
const ACTION_FORCE: f32 = 5000.0;
const POLE_SIZE_HALF: (f32, f32) = (0.1, 2.0);
const CART_SIZE_HALF: (f32, f32) = (2.0, 1.0);
const CART_MASS_DENSITY: f32 = 2.0;
const POLE_MASS_DENSITY: f32 = 0.7;

fn update_pg(world: &mut World) {
    // Find our observables
    let mut cart_pos_x = 0.0;
    let mut cart_vel = 0.0;
    let mut pole_angle = 0.0;
    let mut pole_angle_vel = 0.0;

    // for each cart
    let mut carts = world.query_filtered::<(&RigidBodyPosition, &RigidBodyVelocity), With<Cart>>();
    for (rb_pos, rb_vel) in carts.iter_mut(world) {
        cart_pos_x = rb_pos.position.translation.x;
        cart_vel = rb_vel.linvel[0];
    }

    // for each pole
    let mut poles = world.query_filtered::<(&RigidBodyPosition, &RigidBodyVelocity), With<Pole>>();
    for (rb_pos, rb_vel) in poles.iter(world) {
        pole_angle = rb_pos.position.rotation.angle();
        pole_angle_vel = rb_vel.angvel;
    }
    let observations = vec![cart_pos_x, cart_vel, pole_angle, pole_angle_vel];
    
    // Update state if needed
    let mut state = world.get_resource_mut::<State<CartpoleState>>().unwrap();
    let done = reset_check(cart_pos_x, pole_angle);
    if done {
        state.set(CartpoleState::Resetting).unwrap();
    }

    // Using our observations get an action
    let mut pg = world
        .get_non_send_resource_mut::<PolicyGradientModel>()
        .unwrap();
    let action = tch::no_grad(|| {
        Tensor::of_slice(&observations)
            .unsqueeze(0)
            .apply(&pg.model)
            .softmax(1, Kind::Float)
            .multinomial(1, true)
    });
    let action = f32::from(action);
    
    println!("action: {}", action);

    // Save history
    pg.record_history(observations, 1.0, done, action);

    // Apply action
    let dt = world.get_resource::<IntegrationParameters>().unwrap().dt;
    let mut cart_forces = world.query_filtered::<&mut RigidBodyForces, With<Cart>>();
    for mut rb_f in cart_forces.iter_mut(world) {
        if action < 0.33 {
            rb_f.force = Vec2::new(-ACTION_FORCE * dt, 0.0).into();
        }
        if action > 0.66 {
            rb_f.force = Vec2::new(ACTION_FORCE * dt, 0.0).into();
        }
    }
}

fn check_pg(world: &mut World) {
    let mut pg = world
        .get_non_send_resource_mut::<PolicyGradientModel>()
        .unwrap();

    pg.train();
}

fn update_human(
    keyboard_input: Res<Input<KeyCode>>,
    mut carts: Query<(&RigidBodyPosition, &mut RigidBodyForces), With<Cart>>,
    mut poles: Query<&RigidBodyPosition, With<Pole>>,
    params: Res<IntegrationParameters>,
    mut state: ResMut<State<CartpoleState>>,
) {

    let (rb_pos, mut rb_f) = carts.single_mut()
        .expect("There should be a cart");

    let cart_pos_x = rb_pos.position.translation.x;

    // Take an action
    if keyboard_input.pressed(KeyCode::A) {
        rb_f.force = Vec2::new(-ACTION_FORCE * params.dt, 0.0).into();
    }
    if keyboard_input.pressed(KeyCode::D) {
        rb_f.force = Vec2::new(ACTION_FORCE * params.dt, 0.0).into();
    }

    let pole_rb_pos =  poles.single_mut()
        .expect("There should be a pole");
    let pole_angle = pole_rb_pos.position.rotation.angle();

    if reset_check(cart_pos_x, pole_angle) {
        state.set(CartpoleState::Resetting).unwrap();
    }
}

fn reset_check(cart_pos_x: f32, pole_angle: f32) -> bool {
    if cart_pos_x.abs() > CART_RANGE {
        return true;
    }
    if pole_angle.abs() > POLE_ANGLE_LIMIT {
        return true;
    }
    false
}

fn setup_environment(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    config: Res<EnvironmentConfig>,
    camera: Query<&Camera>,
    mut state: ResMut<State<CartpoleState>>,
) {
    rapier_config.scale = RAPIER_SCALE;

    // Create Camera if needed
    if config.render && camera.iter().count() == 0 {
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
        .insert(CartPoleClean)
        .id();

    // Create Cart
    let cart = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Sensor,
            shape: ColliderShape::cuboid(CART_SIZE_HALF.0, CART_SIZE_HALF.1),
            mass_properties: ColliderMassProps::Density(CART_MASS_DENSITY),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::MAROON))
        .insert(Cart)
        .insert(CartPoleClean)
        .id();

    let mut cart_rollers_joint = PrismaticJoint::new(
        Vec2::new(-50.0, 0.0).into(),
        Vector::x_axis(),
        Vec2::ZERO.into(),
        Vector::x_axis(),
    );
    cart_rollers_joint.limits = [0.0, 100.0];

    commands
        .spawn()
        .insert(JointBuilderComponent::new(cart_rollers_joint, ground, cart))
        .insert(CartPoleClean);

    let mut rnd = rand::thread_rng();

    // Create Pole
    let pole = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, POLE_SIZE_HALF.1 + CART_SIZE_HALF.1).into(),
            // Adding random velocity so its not stable
            velocity: RigidBodyVelocity {
                linvel: Vec2::new(
                    rnd.gen_range(-POLE_INIT_FORCE_LIMIT..POLE_INIT_FORCE_LIMIT),
                    0.0,
                )
                .into(),
                angvel: 0.0,
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(POLE_SIZE_HALF.0, POLE_SIZE_HALF.1),
            collider_type: ColliderType::Sensor,
            mass_properties: ColliderMassProps::Density(POLE_MASS_DENSITY),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::SILVER))
        .insert(Pole)
        .insert(CartPoleClean)
        .id();

    commands.spawn().insert(JointBuilderComponent::new(
        BallJoint::new(
            Vec2::new(0.0, CART_SIZE_HALF.1).into(),
            Vec2::new(0.0, -POLE_SIZE_HALF.1).into(),
        ),
        cart,
        pole,
    ))
    .insert(CartPoleClean);

    // Begin Playing
    state.set(CartpoleState::Playing).unwrap();
}

fn clean_environment(
    mut commands: Commands,
    cleanup: Query<Entity, With<CartPoleClean>>,
    mut state: ResMut<State<CartpoleState>>,
) {
    for e in cleanup.iter() {
        commands.entity(e).despawn();
    }

    state.set(CartpoleState::Loading).unwrap();
}