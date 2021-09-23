use crate::{environment::*, models::neat::NeatML};
use bevy::{ecs::component::Component, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::Rng;
pub struct FlappyPlugin {
    pub render: bool,
}

impl Plugin for FlappyPlugin {
    fn build(&self, app: &mut AppBuilder) {

        let model = NeatML::new("./params/flappy.toml", true );
        
        app.add_startup_system(setup_environment.system())
            .insert_resource(model)
            .add_system(scroll_tubes.system())
            .add_system(catchup_bird.system())
            .add_system_to_stage(CoreStage::First, update_current_tube.system())
            .add_system_to_stage(CoreStage::PreUpdate, update_state.system())
            .add_system_to_stage(CoreStage::Update, update_action.exclusive_system())
            .add_system_to_stage(CoreStage::PostUpdate, take_action.system())
            .add_system(reset_listener.system());

        if self.render {
            app.add_system(keyboard_input.system());
        }
    }
}

fn keyboard_input(mut env_state: ResMut<EnvironmentState>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        env_state.action = Some(0);
    }
}

// Makers to identify entities
struct Tube(TubeType);
struct TubeCurrent;
struct Bird {
    index: usize,
}

// Marking tubes so on respawn can spawn them as pair
enum TubeType {
    Top,
    Bottom,
}

const RAPIER_SCALE: f32 = 50.0; // Very useful to zoom in and out to see whats going on
                                // Also see https://rapier.rs/docs/user_guides/bevy_plugin/common_mistakes/#why-is-everything-moving-in-slow-motion

const TUBE_SIZE_HALF_X: f32 = 1.0;
const TUBE_SIZE_HALF_Y: f32 = 10.0;
const TUBE_DESPAWN_LIMIT: f32 = -15.0;
const TUBE_SPACING: f32 = 10.0;
const TUBE_GAP_SIZE_HALF: f32 = 3.0; // Control gap size between tubes in a set
const TUBE_GAP_OFFSET_MAX: f32 = 4.0; // Control gap range off of y axis
const TUBE_SPEED: f32 = 0.1;
const TUBE_COUNT: usize = 5;
const BIRD_SIZE_HALF: f32 = 0.5;
const BIRD_RESET_LIMIT_Y: f32 = -10.0;
const BIRD_RESET_LIMIT_X: f32 = -4.0;
const ACTION_FORCE: f32 = 150.0;

// Update Current State of the environment
fn update_state(
    mut state: ResMut<EnvironmentState>,
    birds: Query<&RigidBodyPosition, With<Bird>>,
    tubes: Query<(&RigidBodyPosition, &Tube), With<TubeCurrent>>,
    mut ev_reset: EventWriter<EnvironmentResetEvent>,
    mut counter: Local<f32>,
) {
    // Find our observables
    let mut bird_pos = Vec2::new(0.0, 0.0);
    for rb_pos in birds.iter() {
        bird_pos.x =  rb_pos.position.translation.x;
        bird_pos.y =  rb_pos.position.translation.y;
    }

    // Find the lip on the next tubes we need to navigate
    let mut tube_top_lip = 0.0;
    let mut tube_bottom_lip = 0.0;
    for  (rb_pos, tube) in tubes.iter() {
        let y = rb_pos.position.translation.y;
        match tube.0 {
            TubeType::Top => tube_top_lip = y - TUBE_SIZE_HALF_Y,
            TubeType::Bottom => tube_bottom_lip = y + TUBE_SIZE_HALF_Y,
        }
    }
    let top_offset =  tube_top_lip - (bird_pos.y + BIRD_SIZE_HALF);
    let bot_offset =  (bird_pos.y - BIRD_SIZE_HALF) - tube_bottom_lip;

    // Set our observation
    state.observation = vec![bird_pos.y, top_offset, bot_offset];
    state.reward =  *counter;
    let done = reset_check(bird_pos);
    state.is_done = Some(done);

    if done {
        ev_reset.send(EnvironmentResetEvent);
        *counter = 0.0
    } else {
        *counter += 1.0;
    }
}

fn update_action(world: &mut World) {
    let world_cell = world.cell();

    // Get env state and run it though our model giving us an action
    let mut m = world_cell.get_non_send_mut::<NeatML>().unwrap();
    let mut env_state = world_cell.get_resource_mut::<EnvironmentState>().unwrap();

    let action = m.step(&env_state.observation, env_state.reward,  env_state.is_done.unwrap());
    env_state.action = Some(action);
}

fn take_action(
    mut env_state: ResMut<EnvironmentState>,
    mut bird: Query<&mut RigidBodyVelocity, With<Bird>>,
    params: Res<IntegrationParameters>,
) {
    if let Some(action) = env_state.action {
        for mut rb_vel in bird.iter_mut() {
            match action {
                0 => {
                    rb_vel.linvel = Vec2::new(0.0, ACTION_FORCE * params.dt).into();
                }
                1 => {} // Do nothing
                _ => panic!("action invalid: {}", action),
            }
        }
        // Clear after use for now
        env_state.action = None;
    }
}



fn reset_check(bird_pos: Vec2) -> bool {
    let mut result = false;
    if bird_pos.x < BIRD_RESET_LIMIT_X {
        result = true
    }
    if bird_pos.y < BIRD_RESET_LIMIT_Y {
        result = true
    }
    result
}

fn setup_environment(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    config: Res<EnvironmentConfig>,
) {
    rapier_config.scale = RAPIER_SCALE;

    if config.render {
        let mut camera = OrthographicCameraBundle::new_2d();
        camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 50.0));
        commands.spawn_bundle(camera);
    }

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
        .insert(Bird { index: 0 })
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
// This is a system to move bird back to x=0, so it can be used more than once or twice
fn catchup_bird(mut bird: Query<&mut RigidBodyPosition, With<Bird>>) {
    for mut rb_pos in bird.iter_mut() {
        // poor mans lerp
        let x = rb_pos.position.translation.x;
        if x < 0.0 {
            rb_pos.position.translation.x -= x / 60.0;
        }
    }
}

fn update_current_tube(
    mut commands: Commands,
     query_set: QuerySet<(
            Query<(Entity, &RigidBodyPosition), (With<Tube>, Without<TubeCurrent>)>,
            Query<(Entity, &RigidBodyPosition), With<TubeCurrent>>,
        )>
) {
    // Find next tube and mark it
    for (e, rb_pos) in query_set.q0().iter() {
        if rb_pos.position.translation.x < 0.0 && rb_pos.position.translation.x < TUBE_SPACING {
            commands.entity(e).insert(TubeCurrent);
        }
    }

    // Remove any tubes that should be marked
    for (e, rb_pos) in query_set.q1().iter() {
        if rb_pos.position.translation.x < 0.0 || rb_pos.position.translation.x > TUBE_SPACING {
            commands.entity(e).remove::<TubeCurrent>();
        }
    }
}

// Want to let the models trigger reset, so using events
fn reset_listener(
    mut commands: Commands,
    mut ev_reset: EventReader<EnvironmentResetEvent>,
    query_set: QuerySet<(Query<Entity, With<Bird>>, Query<Entity, With<Tube>>)>,
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
