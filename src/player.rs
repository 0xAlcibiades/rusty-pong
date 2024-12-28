//! Player Module
//!
//! This module implements the player paddle mechanics for the Pong game, including both
//! human-controlled and AI-controlled paddles.

use crate::ball::Ball;
use crate::GameState;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

/// Configuration constants for paddle physics and gameplay
#[derive(Debug, Resource)]
pub struct PaddleConfig {
    /// Movement speed in world units per second
    pub speed: f32,
    /// X-coordinate for left paddle position
    pub left_x: f32,
    /// X-coordinate for right paddle position
    pub right_x: f32,
    /// Total height of the paddle
    pub height: f32,
    /// Depth of the paddle's curve
    pub curve_depth: f32,
    /// Number of segments used to create the curved shape
    pub segments: usize,
    /// Mass of the paddle for physics calculations
    pub mass: f32,
    /// Duration of punch animation in seconds
    pub punch_duration: f32,
    /// Distance paddle moves during punch
    pub punch_distance: f32,
}

impl Default for PaddleConfig {
    fn default() -> Self {
        Self {
            speed: 20.0,
            left_x: -7.65,
            right_x: 7.65,
            height: 2.0,
            curve_depth: 0.3,
            segments: 100,
            mass: 0.1,
            punch_duration: 0.05,
            punch_distance: 0.15,
        }
    }
}

/// Configuration for AI difficulty tuning
#[derive(Debug, Resource)]
pub struct AiConfig {
    /// Time between AI decisions (seconds)
    pub update_rate: f32,
    /// Deadzone for movement to prevent jitter
    pub movement_deadzone: f32,
    /// Offset from center for optimal hit point
    pub hit_point_offset: f32,
    /// Chance to make a prediction error (0.0 - 1.0)
    pub error_chance: f32,
    /// Maximum prediction error amount in world units
    pub max_error: f32,
    /// Chance to completely miss the ball (0.0 - 1.0)
    pub miss_chance: f32,
}

/// Configuration for a challenging AI opponent
///
/// While challenging, this AI is intentionally not perfect:
/// - It can be baited into wrong movements
/// - It occasionally misreads steep angles
/// - It has a slight delay before position adjustments
/// - It sometimes misses extremely fast shots
impl Default for AiConfig {
    fn default() -> Self {
        Self {
            // Time between position adjustments
            // Fast enough to handle most shots, but with enough delay
            // that quick direction changes can catch it out of position
            update_rate: 0.3,

            // Minimum distance before adjusting position
            // Small enough to maintain precise positioning for returns,
            // but creates brief windows where slight misalignments can
            // be exploited
            movement_deadzone: 0.08,

            // Preferred hitting position relative to paddle center
            // Aims slightly off-center to maintain better control,
            // but this creates small gaps near the paddle edges that
            // can be targeted
            hit_point_offset: 0.4,

            // Chance to misread the ball's trajectory
            // Most noticeable when handling steep angles or after
            // multiple bounces, representing the limits of its
            // prediction capabilities
            error_chance: 0.12,

            // Maximum prediction error magnitude
            // When errors occur, they're significant enough to create
            // scoring opportunities if the player is positioned to
            // capitalize on them
            max_error: 1.0,

            // Chance to completely miss the ball
            // Primarily occurs during very fast exchanges or when
            // the ball approaches at extreme angles, simulating
            // the challenge of handling powerful shots
            miss_chance: 0.05,
        }
    }
}

/// Component that identifies which player a paddle belongs to
#[derive(Component, Clone, Debug)]
pub enum Player {
    P1, // Human player (left paddle)
    P2, // AI player (right paddle)
}

/// Represents the current movement state of the AI paddle
#[derive(Debug)]
enum MovementState {
    Idle,
    MovingUp(f32),   // Contains target Y position
    MovingDown(f32), // Contains target Y position
}

/// Component for AI-controlled paddles that simulates human-like input behavior
#[derive(Component, Debug)]
struct AiPaddle {
    /// Timer to control AI decision rate
    update_timer: Timer,
    /// Timer for upward movement duration
    move_up_timer: Timer,
    /// Timer for downward movement duration
    move_down_timer: Timer,
    /// Current movement state
    movement_state: MovementState,
    /// Last predicted intersection point
    last_prediction: Option<f32>,
}

impl Default for AiPaddle {
    fn default() -> Self {
        Self {
            update_timer: Timer::from_seconds(
                AiConfig::default().update_rate,
                TimerMode::Repeating,
            ),
            move_up_timer: Timer::from_seconds(0.0, TimerMode::Once),
            move_down_timer: Timer::from_seconds(0.0, TimerMode::Once),
            movement_state: MovementState::Idle,
            last_prediction: None,
        }
    }
}

/// Component to track paddle punch state and animation
#[derive(Component, Debug)]
struct PunchState {
    /// Timer for punch animation duration
    timer: Timer,
    /// Whether paddle is currently in punch state
    is_punching: bool,
    /// Original x position to return to after punch
    rest_x: f32,
}

impl Default for PunchState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(PaddleConfig::default().punch_duration, TimerMode::Once),
            is_punching: false,
            rest_x: 0.0,
        }
    }
}

/// Calculate the duration needed to move to a target position
fn calculate_movement_duration(
    current_pos: f32,
    target_pos: f32,
    speed: f32,
    min_duration: f32,
    max_duration: f32,
) -> f32 {
    let distance = (target_pos - current_pos).abs();
    let base_duration = distance / speed;

    // Add small random variation for more human-like behavior
    let variation = rand::random::<f32>() * 0.1; // Up to 10% variation
    let duration = base_duration * (1.0 + variation);

    // Clamp duration between minimum and maximum values
    duration.clamp(min_duration, max_duration)
}

/// Predicts where the ball will intersect with a paddle's x-position
fn predict_intersection(ball_pos: Vec2, ball_vel: Vec2, paddle_x: f32) -> Option<f32> {
    // Check if ball is moving toward paddle
    let moving_toward =
        (paddle_x > ball_pos.x && ball_vel.x > 0.0) || (paddle_x < ball_pos.x && ball_vel.x < 0.0);

    if moving_toward {
        // Calculate intersection time and position
        let time = (paddle_x - ball_pos.x) / ball_vel.x;
        let y = ball_pos.y + (ball_vel.y * time);
        Some(y)
    } else {
        None
    }
}

/// System that controls AI paddle movement by simulating human-like input
fn ai_decision_making(
    time: Res<Time>,
    paddle_config: Res<PaddleConfig>,
    ai_config: Res<AiConfig>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut ai_query: Query<(&Transform, &mut AiPaddle)>,
) {
    for (paddle_transform, mut ai) in ai_query.iter_mut() {
        // Update movement timers
        ai.move_up_timer.tick(time.delta());
        ai.move_down_timer.tick(time.delta());

        // Reset movement state if timers are finished
        match ai.movement_state {
            MovementState::MovingUp(_) if ai.move_up_timer.finished() => {
                ai.movement_state = MovementState::Idle;
            }
            MovementState::MovingDown(_) if ai.move_down_timer.finished() => {
                ai.movement_state = MovementState::Idle;
            }
            _ => {}
        }

        if ai.update_timer.tick(time.delta()).just_finished() {
            if let Ok((ball_transform, ball_velocity)) = ball_query.get_single() {
                if let Some(predicted_y) = predict_intersection(
                    ball_transform.translation.truncate(),
                    ball_velocity.linvel,
                    paddle_config.right_x,
                ) {
                    // Decide if we're going to try to hit the ball
                    if rand::random::<f32>() < ai_config.miss_chance {
                        // Intentionally miss by moving in wrong direction
                        let miss_y = if predicted_y > 0.0 { -2.0 } else { 2.0 };
                        let current_y = paddle_transform.translation.y;
                        let diff = miss_y - current_y;

                        if diff.abs() > ai_config.movement_deadzone {
                            let duration = calculate_movement_duration(
                                current_y,
                                miss_y,
                                paddle_config.speed,
                                0.1,
                                0.5,
                            );

                            if diff > 0.0 {
                                ai.movement_state = MovementState::MovingUp(miss_y);
                                ai.move_up_timer
                                    .set_duration(Duration::from_secs_f32(duration));
                                ai.move_up_timer.reset();
                            } else {
                                ai.movement_state = MovementState::MovingDown(miss_y);
                                ai.move_down_timer
                                    .set_duration(Duration::from_secs_f32(duration));
                                ai.move_down_timer.reset();
                            }
                        }
                    } else {
                        // Add potential prediction error
                        let error = if rand::random::<f32>() < ai_config.error_chance {
                            let error_amount = rand::random::<f32>() * ai_config.max_error;
                            if rand::random::<bool>() {
                                error_amount
                            } else {
                                -error_amount
                            }
                        } else {
                            0.0
                        };

                        // Calculate hit point with error and offset
                        let optimal_y = predicted_y
                            + error
                            + if ball_velocity.linvel.y > 0.0 {
                                -ai_config.hit_point_offset
                            } else {
                                ai_config.hit_point_offset
                            };

                        let current_y = paddle_transform.translation.y;
                        let diff = optimal_y - current_y;

                        // Only change movement if difference is significant
                        if diff.abs() > ai_config.movement_deadzone {
                            let duration = calculate_movement_duration(
                                current_y,
                                optimal_y,
                                paddle_config.speed,
                                0.1, // Minimum duration
                                0.5, // Maximum duration
                            );

                            if diff > 0.0 {
                                ai.movement_state = MovementState::MovingUp(optimal_y);
                                ai.move_up_timer
                                    .set_duration(Duration::from_secs_f32(duration));
                                ai.move_up_timer.reset();
                            } else {
                                ai.movement_state = MovementState::MovingDown(optimal_y);
                                ai.move_down_timer
                                    .set_duration(Duration::from_secs_f32(duration));
                                ai.move_down_timer.reset();
                            }
                        }
                    }
                    ai.last_prediction = Some(predicted_y);
                }
            }
        }
    }
}

/// Unified system that handles both human and AI paddle movement
fn paddle_movement(
    config: Res<PaddleConfig>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &Player,
        &mut KinematicCharacterController,
        Option<&AiPaddle>,
        &Transform,
    )>,
) {
    for (player, mut controller, ai, paddle_transform) in query.iter_mut() {
        let mut translation = Vec2::ZERO;
        let move_amount = config.speed * time.delta_secs();

        match (player, ai) {
            // Human player input handling
            (Player::P1, None) => {
                if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
                    translation.y += move_amount;
                }
                if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
                    translation.y -= move_amount;
                }
            }
            // AI player movement
            (Player::P2, Some(ai)) => {
                match ai.movement_state {
                    MovementState::MovingUp(target_y) if !ai.move_up_timer.finished() => {
                        // Stop moving if we've reached or passed the target
                        if paddle_transform.translation.y < target_y {
                            translation.y += move_amount;
                        }
                    }
                    MovementState::MovingDown(target_y) if !ai.move_down_timer.finished() => {
                        // Stop moving if we've reached or passed the target
                        if paddle_transform.translation.y > target_y {
                            translation.y -= move_amount;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        controller.translation = Some(translation);
    }
}

/// System that handles paddle-ball collisions and triggers punch animations
fn handle_paddle_collisions(
    config: Res<PaddleConfig>,
    mut collision_events: EventReader<CollisionEvent>,
    mut paddle_query: Query<(Entity, &mut Transform, &mut PunchState), With<Player>>,
    ball_query: Query<Entity, With<Ball>>,
) {
    let Ok(ball_entity) = ball_query.get_single() else {
        return;
    };

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Skip if neither entity is the ball
            if *e1 != ball_entity && *e2 != ball_entity {
                continue;
            }

            for (paddle_entity, mut transform, mut punch_state) in paddle_query.iter_mut() {
                if (paddle_entity == *e1 || paddle_entity == *e2) && !punch_state.is_punching {
                    punch_state.is_punching = true;
                    punch_state.timer.reset();

                    let punch_direction = if transform.translation.x < 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    transform.translation.x += config.punch_distance * punch_direction;
                    break;
                }
            }
        }
    }
}

/// System to reset paddle position after punch animation
fn update_paddle_punch(
    time: Res<Time>,
    mut paddle_query: Query<(&mut Transform, &mut PunchState)>,
) {
    for (mut transform, mut punch_state) in paddle_query.iter_mut() {
        if punch_state.is_punching {
            punch_state.timer.tick(time.delta());
            if punch_state.timer.finished() {
                transform.translation.x = punch_state.rest_x;
                punch_state.is_punching = false;
            }
        }
    }
}

/// Creates mesh and compound collider for paddle
fn create_paddle_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    config: &PaddleConfig,
) -> (Handle<Mesh>, Vec<(Vec2, f32, Collider)>) {
    let mut compound_collider = vec![];
    let mut all_vertices = vec![];

    // Generate segments for the scoop
    for i in 0..config.segments {
        let vertices = generate_segment_vertices(i, config.segments, config);
        all_vertices.extend(vertices.iter().cloned());

        if let Some(collider) = Collider::convex_hull(&vertices) {
            compound_collider.push((Vec2::ZERO, 0.0, collider));
        }
    }

    // Create the mesh for visualization
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Convert 2D vertices to 3D for mesh
    let vertices_3d: Vec<[f32; 3]> = all_vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();

    // Generate indices for triangulation
    let mut indices = Vec::new();
    for i in 0..config.segments {
        let base = i as u32 * 4;
        // First triangle
        indices.extend_from_slice(&[base, base + 1, base + 2]);
        // Second triangle
        indices.extend_from_slice(&[base, base + 2, base + 3]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices_3d);
    mesh.insert_indices(Indices::U32(indices));

    (meshes.add(mesh), compound_collider)
}

/// Helper function to generate vertices for a segment of the scoop paddle shape
fn generate_segment_vertices(
    index: usize,
    total_segments: usize,
    config: &PaddleConfig,
) -> Vec<Vec2> {
    let segment_height = config.height / (total_segments as f32);
    let y_start = -config.height / 2.0 + (index as f32 * segment_height);
    let y_end = y_start + segment_height;

    // Parabolic curve function for paddle front
    let curve = |y: f32| -> f32 {
        let normalized_y = (y + config.height / 2.0) / config.height;
        config.curve_depth * (4.0 * normalized_y * (1.0 - normalized_y))
    };

    vec![
        Vec2::new(0.0, y_start),            // Back left (flat)
        Vec2::new(curve(y_start), y_start), // Front curved
        Vec2::new(curve(y_end), y_end),     // Front curved
        Vec2::new(0.0, y_end),              // Back right (flat)
    ]
}

/// Creates a paddle entity with all necessary components
fn create_paddle(
    commands: &mut Commands,
    config: &PaddleConfig,
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<ColorMaterial>,
    is_player_one: bool,
    compound_collider: Vec<(Vec2, f32, Collider)>,
) -> Entity {
    let x_pos = if is_player_one {
        config.left_x
    } else {
        config.right_x
    };
    let rotation = if is_player_one {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_z(std::f32::consts::PI)
    };

    let mut entity = commands.spawn_empty();

    // Add visual components
    entity
        .insert(Mesh2d(mesh_handle))
        .insert(MeshMaterial2d(material_handle))
        .insert(Transform::from_xyz(x_pos, 0.0, 0.0).with_rotation(rotation))
        .insert(GlobalTransform::default())
        .insert(Visibility::default())
        .insert(ViewVisibility::default())
        .insert(InheritedVisibility::default());

    // Add physics components
    entity
        .insert(RigidBody::KinematicPositionBased)
        .insert(KinematicCharacterController::default())
        .insert(Collider::compound(compound_collider))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(AdditionalMassProperties::Mass(config.mass));

    // Add player-specific components
    if is_player_one {
        entity.insert(Player::P1);
    } else {
        entity.insert(Player::P2).insert(AiPaddle::default());
    }

    // Add punch state
    entity.insert(PunchState {
        rest_x: x_pos,
        ..default()
    });

    entity.id()
}

/// Spawns both player paddles: human P1 on left and AI P2 on right
fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let config = PaddleConfig::default();

    // Create paddle mesh and collider
    let (mesh_handle, compound_collider) = create_paddle_mesh(&mut meshes, &config);
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    // Spawn player 1 (left paddle)
    create_paddle(
        &mut commands,
        &config,
        mesh_handle.clone(),
        material_handle.clone(),
        true,
        compound_collider.clone(),
    );

    // Spawn player 2 (right paddle)
    create_paddle(
        &mut commands,
        &config,
        mesh_handle,
        material_handle,
        false,
        compound_collider,
    );
}

/// Plugin that manages all player-related systems
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize configuration resources
            .init_resource::<PaddleConfig>()
            .init_resource::<AiConfig>()
            // Add startup systems
            .add_systems(Startup, spawn_players)
            // Add gameplay systems that run during the Playing state
            .add_systems(
                Update,
                (
                    ai_decision_making,
                    paddle_movement,
                    handle_paddle_collisions,
                    update_paddle_punch,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
