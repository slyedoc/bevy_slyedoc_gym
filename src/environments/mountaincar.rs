use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::environment::*;

pub struct MountainCarPlugin {
    pub render: bool,
}
// Makers
struct Cart;
struct Ground;

impl Plugin for MountainCarPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);

        app.add_startup_system(setup_physics.system());

        if self.render {
            app.add_startup_system(setup_graphics.system())
                .add_system(keyboard_input.system());
        }
    }
}

fn setup_graphics(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 100.0));
    commands.spawn_bundle(camera);
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyForces, With<Cart>>,
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

fn setup_physics(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scaling up, see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
    rapier_config.scale = 50.0;


    // To create the ground from list of points following sin curve
    let ground_width = 20.0f32; // assuming at origin
    let ground_resolution = 0.2; // Smoothness, space between vertices

    // uses to change shape of sin curve, changing x will effect resolution
    let ground_scale = Vec3::new(1.0, 2.0, 1.0); 

    let half_segments = (( ground_width * 0.5) / ground_resolution) as i8;
    let mut vertices: Vec<Point<Real>> = Vec::new();
    for i in -half_segments..=half_segments {
        let i = f32::from(i) * ground_resolution;
        let x = i * ground_scale.x;
        let y = x.sin() * ground_scale.y;
        vertices.push(Vec2::new(x, y).into());
    }

    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::polyline(vertices, None),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::BLACK))
        .insert(Ground)
        .id();


    let cart_size = Vec2::new(0.2, 1.0);
    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 10.0).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(cart_size.x, cart_size.y),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::GRAY))
        .insert(Cart)
        .id();

    // TODO: Figure out rollings the cart
}

