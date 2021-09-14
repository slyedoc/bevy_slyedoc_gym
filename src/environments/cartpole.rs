use crate::environment::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
pub struct CartPolePlugin {
    pub render: bool,
}

impl Plugin for CartPolePlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);

        app.add_startup_system(setup_enviroment.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system())
            .add_system(reset_listener.system());

        if self.render {
            app.add_startup_system(setup_graphics.system())
                .add_system(keyboard_input.system());
        }
    }
}

// Makers to identify entities
struct Cart;
struct Pole;
struct Ground;

const CART_RANGE: f32 = 4.8;
const POLE_ANGLE: f32 = 0.418; // 24 degrees
const ACTION_FORCE: f32 = 3000.0; // F / dt

fn take_action(
    mut env_action: ResMut<EnvironmentAction>,
    mut cart: Query<&mut RigidBodyForces, With<Cart>>,
    params: Res<IntegrationParameters>,
) {
    //println!("take_action: {}", env_action.take);
    if env_action.take {
        for mut rb_f in cart.iter_mut() {
            match env_action.action {
                0 => rb_f.force = Vec2::new(-ACTION_FORCE * params.dt , 0.0).into(),
                1 => rb_f.force = Vec2::new(ACTION_FORCE * params.dt , 0.0).into(),
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
    pole: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Pole>>,
    cart: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Cart>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
) {
    //println!("udpate_state");
    // Find our obserables
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
    state.action = env_action.action;
    state.reward = 1.0;
    state.is_done = reset_check(cart_pos_x, pole_angle);
    if state.is_done {
        ev_reset.send(EnvironmentResetEvent);
    }
}

fn reset_check(cart_pos_x: f32, pole_angle: f32) -> bool {
    let mut reset = false;
    if cart_pos_x.abs() > CART_RANGE {
        reset = true;
    }
    if pole_angle.abs() > POLE_ANGLE {
        reset = true;
    }
    reset
}

fn setup_graphics(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 150.0, 50.0));
    commands.spawn_bundle(camera);
}

fn setup_enviroment(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.scale = 50.0;

    /* Create the ground. */
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

    spawn_cartpole(commands, ground);
}

fn spawn_cartpole(mut commands: Commands, ground: Entity) {
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

    let pole = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, (pole_size.y * 0.5) + (cart_size.y * 0.5)).into(),
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

        // TODO: hmmmm, do joints despawn with there entity, if not need to do it here

        let g = query_set.q2().iter().next();
        if let Some(g) = g {
            spawn_cartpole(commands, g);
        }
    }
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyForces, With<Cart>>,
    keyboard_input: Res<Input<KeyCode>>,
    params: Res<IntegrationParameters>,
) {
    for mut rb_forces in rigid_bodies.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            rb_forces.force = Vec2::new(-ACTION_FORCE * params.dt, 0.0).into();
        }
        if keyboard_input.pressed(KeyCode::D) {
            rb_forces.force = Vec2::new(ACTION_FORCE * params.dt, 0.0).into();
        }
    }
}
