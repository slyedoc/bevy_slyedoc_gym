use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::environment::*;
pub struct CartPolePlugin {
    pub render: bool,
}


impl Plugin for CartPolePlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);

        app.init_resource()
            .add_startup_system(setup_physics.system())
            .add_system(update_state.system())
            .add_system(take_action.system())
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

fn take_action(
    env_action: Res<EnvironmentAction>,
    mut cart: Query<&mut RigidBodyForces, With<Cart>>,
) {
    for mut rb_f in cart.iter_mut() {
        match env_action.action {
            0 => rb_f.force = Vec2::new(-100.0, 0.0).into(),
            1 => rb_f.force = Vec2::new(100.0, 0.0).into(),
            _ => panic!("action invalid")
        }
    }
}



// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    pole: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Pole>>,
    cart: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Cart>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
) {
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
    state.observation = vec![cart_pos_x, cart_vel, pole_angle, pole_angle_vel];

    // Set out our reward
    state.reward += 1.0;

    // Check that the cart is with in bounds
    let reset = reset_check(cart_pos_x, pole_angle);
    if reset {
        state.is_done = true;
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
    camera.transform = Transform::from_translation(Vec3::new(0.0, 250.0, 100.0));
    commands.spawn_bundle(camera);
}

fn setup_physics(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
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
    let pole_size = Vec2::new(0.1, 3.0);
    let cart_size = Vec2::new(2.0, 1.0);
    let cart = commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Sensor,
            shape: ColliderShape::cuboid(cart_size.x, cart_size.y),
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
            position: Vec2::new(0.0, pole_size.y * 0.5).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(pole_size.x, pole_size.y),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            },
            collider_type: ColliderType::Sensor,
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
        Vec2::new(0.0, cart_size.y).into(),
        Vec2::new(0.0, -pole_size.y).into(),
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
        println!("Reset");
        reset = true;
    }
    if reset {

        for e in query_set.q0().iter() {
            commands.entity(e).despawn_recursive();
        }
        for e in query_set.q1().iter() {
            commands.entity(e).despawn_recursive();
        }

        let g =  query_set.q2().iter().next();
        if let Some(g) = g {
            spawn_cartpole(commands, g);
        }
    }
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyForces, With<Cart>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut rb_forces in rigid_bodies.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            rb_forces.force = Vec2::new(-100.0, 0.0).into();
        }
        if keyboard_input.pressed(KeyCode::D) {
            rb_forces.force = Vec2::new(100.0, 0.0).into();
        }
    }
}
