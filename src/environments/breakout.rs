use crate::{
    helpers::{range_lerp, V2},
    models::neat::NeatML,
};
use bevy::{ecs::component::Component, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::Rng;
use std::ops::Range;

const RAPIER_SCALE: f32 = 50.0;
const PLAYER_SIZE_HALF: V2<f32> = V2 { x: 1.0, y: 0.1 };
const PLAYER_SPEED: f32 = 200.0;
const PLAYER_COLOR: Color = Color::BLUE;
const BOARD_SIZE_HALF: V2<f32> = V2 { x: 4.0, y: 6.0 };
const BOARD_LINE_SIZE_HALF: f32 = 0.1;
const BOARD_COLOR: Color = Color::BLUE;
const BRICK_GRID: V2<usize> = V2 { x: 4, y: 2 };
const BALL_SIZE_HALF: f32 = 0.1;
const BALL_INIT_X_RANGE: Range<f32> = -1.0..1.0;
const BALL_INIT_Y: f32 = 5.0;

#[derive(Copy, Clone)]
pub struct BreakoutConfig {
    pub render: bool,
    pub human: bool,
}

struct Brick;
struct BrickDespawn;
struct Player {
    index: usize,
}
struct Score(usize);
struct Ball;

struct BoardBottom;
struct BoardOther;

struct BreakoutCleanup;

pub struct BreakoutPlugin {
    pub config: BreakoutConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BreakoutState {
    Loading,
    Playing,
    Resetting,
}

impl Plugin for BreakoutPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(self.config)
            .insert_resource(Score(0))
            .add_state(BreakoutState::Loading)
            .add_system_set(
                SystemSet::on_enter(BreakoutState::Loading)
                    .with_system(setup_environment.system())
                    .with_system(spawn_board.system())
                    .with_system(spawn_player.system())
                    .with_system(spawn_ball.system()),
            )
            .add_system_set(
                SystemSet::on_update(BreakoutState::Playing)
                    .with_system(update_ball.system())
                    .with_system(ball_collision.system())
                    .with_system(despawn_bricks.system())
                    .with_system(keyboard_input_system.system())
                    .with_system(ball_bounds_check.system()),
            )
            .add_system_set(
                SystemSet::on_enter(BreakoutState::Resetting)
                    .with_system(clean_environment.system()),
            );

        if self.config.human {
            app.add_system_set(
                SystemSet::on_update(BreakoutState::Playing)
                    .with_system(player_movement_human.system()),
            );
            println!("Press A or D, or Left or Right Arrow\nR to reset\nEscape to exit");
        } else {
            app.add_system_set(
                SystemSet::on_update(BreakoutState::Playing)
                    .with_system(player_movement_neat.system()),
            );
            println!("Press Escape to exit");
        }
    }
}

fn setup_environment(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    config: ResMut<BreakoutConfig>,
    mut state: ResMut<State<BreakoutState>>,
    mut score: ResMut<Score>,
) {
    rapier_config.scale = RAPIER_SCALE;
    rapier_config.gravity = Vec2::ZERO.into();
    score.0 = 0;

    if config.render {
        let mut camera = OrthographicCameraBundle::new_2d();
        camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 50.0));
        commands.spawn_bundle(camera).insert(BreakoutCleanup);
    }
    state.set(BreakoutState::Playing).unwrap();
}

fn spawn_board(mut commands: Commands) {
    // draw board
    commands
        .spawn_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .with_children(|parent| {
            // Top
            create_board_side(
                parent,
                Vec2::new(0.0, BOARD_SIZE_HALF.y),
                Vec2::new(
                    BOARD_SIZE_HALF.x + BOARD_LINE_SIZE_HALF,
                    BOARD_LINE_SIZE_HALF,
                ),
                BoardOther,
            );
            // Bottom
            create_board_side(
                parent,
                Vec2::new(0.0, -BOARD_SIZE_HALF.y),
                Vec2::new(
                    BOARD_SIZE_HALF.x + BOARD_LINE_SIZE_HALF,
                    BOARD_LINE_SIZE_HALF,
                ),
                BoardBottom,
            );
            // Left
            create_board_side(
                parent,
                Vec2::new(-BOARD_SIZE_HALF.x, 0.0),
                Vec2::new(BOARD_LINE_SIZE_HALF, BOARD_SIZE_HALF.y),
                BoardOther,
            );
            // Right
            create_board_side(
                parent,
                Vec2::new(BOARD_SIZE_HALF.x, 0.0),
                Vec2::new(BOARD_LINE_SIZE_HALF, BOARD_SIZE_HALF.y),
                BoardOther,
            );

            let size_x: f32 = BOARD_SIZE_HALF.x / (BRICK_GRID.x + 2) as f32;
            let size_y: f32 = BOARD_SIZE_HALF.y * 0.5 / (BRICK_GRID.y + 2) as f32;
            // Create Bricks
            for x in 0..BRICK_GRID.x {
                for y in 0..BRICK_GRID.y {
                    let pos_x = range_lerp(
                        (x + 1) as f32,
                        0.0,
                        (BRICK_GRID.x + 1) as f32,
                        -BOARD_SIZE_HALF.x,
                        BOARD_SIZE_HALF.x,
                    );
                    let pos_y = range_lerp(
                        (y + 1) as f32,
                        0.0,
                        (BRICK_GRID.y + 1) as f32,
                        0.0,
                        BOARD_SIZE_HALF.y,
                    );

                    create_brick(
                        parent,
                        Vec2::new(pos_x, pos_y),
                        Vec2::new(size_x, size_y),
                        Color::rgb_linear(0.0, 0.0, 0.0),
                    );
                }
            }
        })
        .insert(ColliderDebugRender::default())
        .insert(ColliderPositionSync::Discrete);
}

fn create_board_side(
    parent: &mut ChildBuilder,
    pos: Vec2,
    size_half: Vec2,
    component: impl Component,
) {
    parent
        .spawn_bundle(ColliderBundle {
            position: pos.into(),
            collider_type: ColliderType::Solid,
            shape: ColliderShape::cuboid(size_half.x, size_half.y),
            material: ColliderMaterial {
                friction: 0.0,
                restitution: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(BOARD_COLOR))
        .insert(BreakoutCleanup)
        .insert(component);
}

fn create_brick(parent: &mut ChildBuilder, pos: Vec2, size_half: Vec2, color: Color) {
    parent
        .spawn_bundle(ColliderBundle {
            position: pos.into(),
            collider_type: ColliderType::Solid,
            material: ColliderMaterial {
                friction: 0.0,
                restitution: 1.0,
                ..Default::default()
            },
            shape: ColliderShape::cuboid(size_half.x, size_half.y),
            flags: (ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(color))
        .insert(Brick)
        .insert(BreakoutCleanup);
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, -BOARD_SIZE_HALF.y + (BOARD_SIZE_HALF.y * 0.1)).into(),
            mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::cuboid(PLAYER_SIZE_HALF.x, PLAYER_SIZE_HALF.y),
            material: ColliderMaterial {
                restitution: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(PLAYER_COLOR))
        .insert(Player { index: 0 })
        .insert(BreakoutCleanup);
}

fn spawn_ball(mut commands: Commands) {
    let mut rnd = rand::thread_rng();

    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, -BOARD_SIZE_HALF.y + (BOARD_SIZE_HALF.y * 0.2)).into(),
            mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED).into(),
            activation: RigidBodyActivation::cannot_sleep(),
            ccd: RigidBodyCcd {
                ccd_enabled: true,
                ..Default::default()
            },
            damping: RigidBodyDamping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            // Create random launch vector
            velocity: RigidBodyVelocity {
                linvel: Vec2::new(rnd.gen_range(BALL_INIT_X_RANGE), BALL_INIT_Y).into(),
                angvel: 0.0,
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::ball(BALL_SIZE_HALF),
            flags: (ActiveEvents::CONTACT_EVENTS).into(),
            material: ColliderMaterial {
                friction: 0.0, // you lose all ball control on paddle at 0
                restitution: 1.0,
                restitution_combine_rule: CoefficientCombineRule::Max,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::from(PLAYER_COLOR))
        .insert(Ball)
        .insert(BreakoutCleanup);
}

// Keep the ball speed somewhat constant and  avoid getting stuck by back and forth
fn update_ball(mut balls: Query<&mut RigidBodyVelocity, With<Ball>>) {
    for mut rb_vel in balls.iter_mut() {
        // Normalize ball speed, currently picked at random
        let mag = rb_vel.linvel.norm();
        if mag < 5.0 || mag > 6.0 {
            rb_vel.linvel *= 5.0 / mag;
            //warn!("ball speed, mag: {}", mag);
        }

        // This will curve that ball up when its going more left to right that up and down
        // so it can't get stuck, relies on the speed normalizing above
        if rb_vel.linvel[0].abs() > 4.0 {
            rb_vel.linvel[1] += if rb_vel.linvel[1].is_sign_negative() {
                -0.01
            } else {
                0.01
            };
            //warn!("curve ball, {}", rb_vel.linvel[0].abs());
        }
    }
}

// The ball can get away using the paddle to force it though a wall, this checks for that
fn ball_bounds_check(
    balls: Query<&RigidBodyPosition, With<Ball>>,
    mut state: ResMut<State<BreakoutState>>,
) {
    for rb_pos in balls.iter() {
        if rb_pos.position.translation.x.abs() > BOARD_SIZE_HALF.x
            || rb_pos.position.translation.y.abs() > BOARD_SIZE_HALF.y
        {
            state.set(BreakoutState::Resetting).unwrap()
        }
    }
}

fn ball_collision(
    mut commands: Commands,
    narrow_phase: Res<NarrowPhase>,
    bricks: Query<Entity, With<Brick>>,
    balls: Query<Entity, With<Ball>>,
    board_bottom: Query<Entity, With<BoardBottom>>,
    mut score: ResMut<Score>,
    mut state: ResMut<State<BreakoutState>>,
) {
    let ball = balls.single().expect("Should be one ball at least");
    for brick in bricks.iter() {
        // Find the contact pair, if it exists, between two colliders.
        if let Some(contact) = narrow_phase.contact_pair(ball.handle(), brick.handle()) {
            if contact.has_any_active_contact {
                // We have a contact, mark for despawn, using event delay
                commands.entity(brick).insert(BrickDespawn);
                score.0 += 1;

                // is that all of them?
                if score.0 == BRICK_GRID.x * BRICK_GRID.y {
                    state.set(BreakoutState::Resetting).unwrap();
                    return;
                }
            }
        }

        let bottom = board_bottom.single().expect("More than one bottom?");
        if let Some(contact) = narrow_phase.contact_pair(ball.handle(), bottom.handle()) {
            if contact.has_any_active_contact {
                state.set(BreakoutState::Resetting).unwrap();
                return;
            }
        }
    }
}

// Removing bricks after collision
fn despawn_bricks(mut commands: Commands, mut bricks: Query<Entity, With<BrickDespawn>>) {
    for e in bricks.iter_mut() {
        commands.entity(e).despawn();
    }
}

fn keyboard_input_system(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<BreakoutState>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        state.set(BreakoutState::Resetting).unwrap();
        // TODO: You get stuck in a loop without updating keyboard
        // https://github.com/bevyengine/bevy/issues/1700
        keyboard_input.update();
    }
}

fn player_movement_human(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<&mut RigidBodyVelocity, With<Player>>,
    params: Res<IntegrationParameters>,
) {
    for mut rb_vel in players.iter_mut() {
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
        if left {
            rb_vel.linvel = Vec2::new(-PLAYER_SPEED * params.dt, 0.0).into();
        } else if right {
            rb_vel.linvel = Vec2::new(PLAYER_SPEED * params.dt, 0.0).into();
        } else {
            rb_vel.linvel = Vec2::new(0.0, 0.0).into();
        }
    }
}

fn player_movement_neat(
    mut neat: ResMut<NeatML>,
    mut players: Query<(&Player, &RigidBodyPosition, &mut RigidBodyVelocity)>,
    balls: Query<&RigidBodyPosition, With<Ball>>,
    params: Res<IntegrationParameters>,
) {
    let ball_pos = balls.single().expect("One ball should exist!");

    for (player, pos, mut rb_vel) in players.iter_mut() {
        let observations = &[
            pos.position.translation.x as f64,      // player x
            pos.position.translation.y as f64,      // player y
            ball_pos.position.translation.x as f64, // ball x
            ball_pos.position.translation.y as f64, // ball y
        ];
        let output = neat.pool.activate_nth(player.index, observations).unwrap();

        // Go left
        rb_vel.linvel = match output[0] {
            o if o < 0.33 => Vec2::new(-PLAYER_SPEED * params.dt, 0.0).into(),
            o if o > 0.66 => Vec2::new(PLAYER_SPEED * params.dt, 0.0).into(),
            _ => Vec2::ZERO.into(),
        }
    }
}

fn clean_environment(
    mut commands: Commands,
    cleanup: Query<Entity, With<BreakoutCleanup>>,
    mut state: ResMut<State<BreakoutState>>,
) {
    for e in cleanup.iter() {
        commands.entity(e).despawn();
    }
    state.set(BreakoutState::Loading).unwrap();
}
