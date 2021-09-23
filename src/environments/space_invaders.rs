use crate::environment::*;
use bevy::{ecs::component::Component, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::Rng;
pub struct SpaceInvadersPlugin {
    pub render: bool,
}

impl Plugin for SpaceInvadersPlugin {
    fn build(&self, app: &mut AppBuilder) {
        insert_env_resources(app, 2, 4);

        app.add_startup_system(setup_environment.system())
            .add_system(scroll_tubes.system())
            .add_system(catchup_bird.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system())
            .add_system(reset_listener.system());

        if self.render {
            app.add_startup_system(setup_camera.system())
                .add_system(keyboard_input.system());
        }
    }
}

// Makers to identify entities
struct Tube(TubeType);
struct Bird;

// Marking tubes so on respawn can spawn them as pair
enum TubeType {
    Top,
    Bottom,
}

const TUBE_SIZE_HALF_X: f32 = 1.0;
const TUBE_SIZE_HALF_Y: f32 = 10.0;
const TUBE_DESPAWN_LIMIT: f32 = -15.0;
const TUBE_SPACING: f32 = 10.0;
const TUBE_GAP_SIZE_HALF: f32 = 3.0;  // Control gap size between tubes in a set
const TUBE_GAP_OFFSET_MAX: f32 = 4.0; // Control gap range off of y axis
const TUBE_SPEED: f32 = 0.1;
const TUBE_COUNT: usize = 5;
const BIRD_SIZE_HALF: f32 = 0.5;
const BIRD_RESET_LIMIT_Y: f32 = -10.0;
const BIRD_RESET_LIMIT_X: f32 = -4.0;
const ACTION_FORCE: f32 = 150.0; // F / dt

fn take_action(
    mut env_state: ResMut<EnvironmentState>,
    mut bird: Query<&mut RigidBodyVelocity, With<Bird>>,
    _params: Res<IntegrationParameters>,
) {
    if let Some(action) = env_state.action {
        for _rb_vel in bird.iter_mut() {
            match action {
                0 => {} //action_up(rb_vel, &params),
                1 => {} // Do nothing
                _ => panic!("action invalid: {}", action),
            }
        }
        env_state.action = None;
    }
}

fn action_up(mut rb_vel: Mut<RigidBodyVelocity>, params: &Res<IntegrationParameters>) {
    rb_vel.linvel = Vec2::new(0.0, ACTION_FORCE * params.dt).into();
}

// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    bird: Query<(&RigidBodyPosition, &RigidBodyVelocity), With<Bird>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
) {
    // Find our observables
    let mut bird_pos = Vec2::new(0.0, 0.0);
    let mut bird_vel = 0.0;
    for (rb_pos, rb_vel) in bird.iter() {
        bird_pos.x = rb_pos.position.translation.x;
        bird_pos.y = rb_pos.position.translation.y;
        bird_vel = rb_vel.linvel[0];
    }

    // TODO: Update state using that info
    state.observation = vec![bird_pos.x, bird_vel];

    // state.reward = 1.0;
    state.is_done = reset_check(bird_pos);
    if state.is_done {
        ev_reset.send(EnvironmentResetEvent);
    }
}

fn reset_check(bird_pos: Vec2) -> bool {
    let mut result = false;
    if bird_pos.x < BIRD_RESET_LIMIT_X {
        result = true
    }
    if bird_pos.y < BIRD_RESET_LIMIT_Y  {
        result = true
    }
    result
}

fn setup_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 50.0));
    commands.spawn_bundle(camera);
}

fn setup_environment(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    // Very useful to zoom in and out to see whats going on
    // Also see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion
    rapier_config.scale = 50.0;

    spawn_env(&mut commands);
}

// Assumes clean slate, called at startup and on reset events
fn spawn_env(commands: &mut Commands) {
    // Create the Bird
    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(BIRD_SIZE_HALF, BIRD_SIZE_HALF),
            collider_type: ColliderType::Solid,
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::RED))
        .insert(Bird)
        .id();

    // setup tubes
    for x in 0..TUBE_COUNT {
        spawn_tube_set(commands, x as f32 * TUBE_SPACING);
    }
}

fn spawn_tube_set(commands: &mut Commands, pos_x: f32) {
    // figure out where the tubes should be
    let mut rng = rand::thread_rng();
    let gap_offset = rng.gen_range(-TUBE_GAP_OFFSET_MAX..TUBE_GAP_OFFSET_MAX);
    let spacing = TUBE_SIZE_HALF_Y + TUBE_GAP_SIZE_HALF;

    //println!("spacing: {}, gap_offset: {}", spacing, gap_offset);
    spawn_tube(
        commands,
        Vec2::new(pos_x, spacing + gap_offset),
        Tube(TubeType::Top),
    );
    spawn_tube(
        commands,
        Vec2::new(pos_x, -spacing + gap_offset),
        Tube(TubeType::Bottom),
    );
}

fn spawn_tube(commands: &mut Commands, pos: Vec2, tube_type: impl Component) -> Entity {
    commands
        .spawn_bundle(RigidBodyBundle {
            position: pos.into(),
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::cuboid(TUBE_SIZE_HALF_X, TUBE_SIZE_HALF_Y),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(Color::GREEN))
        .insert(tube_type)
        .id()
}

fn scroll_tubes(
    mut commands: Commands,
    mut tube_set: Query<(Entity, &mut RigidBodyPosition, &Tube)>,
) {
    for (e, mut rb_pos, tube) in tube_set.iter_mut() {
        rb_pos.position.translation.x -= TUBE_SPEED;

        // despawn when off screen and spawn new tube
        if rb_pos.position.translation.x < TUBE_DESPAWN_LIMIT {
            commands.entity(e).despawn();

            match tube.0 {
                TubeType::Top => spawn_tube_set(
                    &mut commands,
                    TUBE_COUNT as f32 * TUBE_SPACING + TUBE_DESPAWN_LIMIT,
                ),
                TubeType::Bottom => {}
            }
        }
    }
}

// The bird can get stuck behind tubes and falls behind and skid off top of tubes
// I could just trigger game over when a touch occurs but I like the effect
// This is a system move bird back to x=0, so it can be used more than once or twice
fn catchup_bird(
    mut bird: Query<&mut RigidBodyPosition, With<Bird>>,
) {
    for mut rb_pos in bird.iter_mut() {
        // poor mans lerp
        let x = rb_pos.position.translation.x;
        if x < 0.0 {
            rb_pos.position.translation.x -= x / 60.0;
        }
    }
}

// Want to let the models trigger reset, so using events
fn reset_listener(
    mut commands: Commands,
    mut ev_reset: EventReader<EnvironmentResetEvent>,
    query_set: QuerySet<(Query<Entity, With<Bird>>, Query<Entity, With<Tube>>)>
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

        spawn_env(&mut commands);
    }
}

fn keyboard_input(
    mut rigid_bodies: Query<&mut RigidBodyVelocity, With<Bird>>,
    keyboard_input: Res<Input<KeyCode>>,
    params: Res<IntegrationParameters>,
) {
    for rb_vel in rigid_bodies.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            action_up(rb_vel, &params);
        }
    }
}
