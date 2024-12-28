//! Player Module
//!
//! This module handles both human and AI-controlled paddles, including:
//! - Paddle physics and movement
//! - Player input handling
//! - AI behavior and decision making
//! - Ball collision and bounce mechanics
//!
//! The module supports both human input (Player 1) and AI control (Player 2).

use crate::ball::Ball;
use crate::GameState;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier2d::prelude::*;

/// Constants defining the player paddles' physical and gameplay properties
const PADDLE_SPEED: f32 = 20.0; // Movement speed in units per second
const MAX_BOUNCE_ANGLE: f32 = 1.0; // Maximum angle ball can bounce (radians)
const LEFT_PADDLE_X: f32 = -7.65; // X position of left paddle
const RIGHT_PADDLE_X: f32 = 7.65; // X position of right paddle

// Define scoop paddle shape constants
const PADDLE_WIDTH: f32 = 0.5; // Base width of the paddle
const PADDLE_HEIGHT: f32 = 2.0; // Height of the paddle
const PADDLE_DEPTH: f32 = 0.3; // How deep the curve goes
const SEGMENTS: usize = 100; // Number of segments to create the curve

/// Update rate for AI decisions to simulate human-like reaction time
const AI_UPDATE_RATE: f32 = 0.3; // Updates 3.33 times per second

/// Component that identifies which player a paddle belongs to
#[derive(Component, Clone)]
pub enum Player {
    P1, // Human player (left paddle)
    P2, // AI player (right paddle)
}

/// Component for AI-controlled paddles that simulates human-like input behavior
#[derive(Component)]
struct AiPaddle {
    update_timer: Timer, // Controls AI decision rate
    move_up: bool,       // Simulates W key press
    move_down: bool,     // Simulates S key press
}

/// Component to track paddle punch state
#[derive(Component)]
struct PunchState {
    timer: Timer,
    is_punching: bool,
    rest_x: f32, // The x position to return to
}

impl Default for AiPaddle {
    fn default() -> Self {
        Self {
            update_timer: Timer::from_seconds(AI_UPDATE_RATE, TimerMode::Repeating),
            move_up: false,
            move_down: false,
        }
    }
}

impl Default for PunchState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.05, TimerMode::Once), // 50ms punch duration
            is_punching: false,
            rest_x: 0.0, // Will be set on spawn
        }
    }
}

/// Helper function to generate vertices for a segment of the scoop paddle shape
/// Creates a paddle with a flat back and curved front
///
/// # Arguments
/// * `index` - Index of the current segment being generated
/// * `total_segments` - Total number of segments in the paddle
///
/// # Returns
/// * Vector of Vec2 points defining a convex segment of the paddle
fn generate_segment_vertices(index: usize, total_segments: usize) -> Vec<Vec2> {
    let segment_height = PADDLE_HEIGHT / (total_segments as f32);
    let y_start = -PADDLE_HEIGHT / 2.0 + (index as f32 * segment_height);
    let y_end = y_start + segment_height;

    // Calculate x position for the curved front using a parabolic curve
    // The curve bulges outward from the flat back
    let curve = |y: f32| -> f32 {
        let normalized_y = (y + PADDLE_HEIGHT / 2.0) / PADDLE_HEIGHT;
        // Start from left side and curve outward
        PADDLE_DEPTH * (4.0 * normalized_y * (1.0 - normalized_y))
    };

    vec![
        Vec2::new(0.0, y_start),            // Back left (flat)
        Vec2::new(curve(y_start), y_start), // Front curved
        Vec2::new(curve(y_end), y_end),     // Front curved
        Vec2::new(0.0, y_end),              // Back right (flat)
    ]
}

/// Calculates where the ball will intersect with a paddle's x-position
///
/// # Arguments
/// * `ball_pos` - Current ball position
/// * `ball_vel` - Current ball velocity
/// * `paddle_x` - X-coordinate of paddle to check
///
/// # Returns
/// * `Some(y)` - Predicted Y intersection if ball is moving toward paddle
/// * `None` - If ball is moving away from paddle
fn predict_intersection(ball_pos: Vec2, ball_vel: Vec2, paddle_x: f32) -> Option<f32> {
    // Only predict if ball is moving toward the paddle
    if (paddle_x > ball_pos.x && ball_vel.x > 0.0) || (paddle_x < ball_pos.x && ball_vel.x < 0.0) {
        // Calculate time until x-intersection
        let time = (paddle_x - ball_pos.x) / ball_vel.x;
        // Calculate y-position at intersection
        let y = ball_pos.y + (ball_vel.y * time);
        Some(y)
    } else {
        None
    }
}

/// System that controls AI paddle movement by simulating human-like input
///
/// The AI:
/// 1. Predicts ball intersection point
/// 2. Updates movement decision at human-like intervals
/// 3. Moves paddle toward predicted position
fn ai_decision_making(
    time: Res<Time>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut ai_query: Query<(&Transform, &mut AiPaddle)>,
) {
    for (paddle_transform, mut ai) in ai_query.iter_mut() {
        if ai.update_timer.tick(time.delta()).just_finished() {
            if let Ok((ball_transform, ball_velocity)) = ball_query.get_single() {
                if let Some(predicted_y) = predict_intersection(
                    ball_transform.translation.truncate(),
                    ball_velocity.linvel,
                    RIGHT_PADDLE_X,
                ) {
                    // Add offset to account for curved surface optimal hit point
                    let optimal_y = predicted_y
                        + (if ball_velocity.linvel.y > 0.0 {
                            -PADDLE_HEIGHT / 4.0
                        } else {
                            PADDLE_HEIGHT / 4.0
                        });

                    // Calculate difference between current and target position
                    let diff = optimal_y - paddle_transform.translation.y;

                    // Reset movement flags
                    ai.move_up = false;
                    ai.move_down = false;

                    // Move toward predicted position with small deadzone
                    if diff > 0.1 {
                        ai.move_up = true;
                    } else if diff < -0.1 {
                        ai.move_down = true;
                    }
                }
            }
        }
    }
}

/// Unified system that handles both human and AI paddle movement
fn paddle_movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &Player,
        &mut KinematicCharacterController,
        Option<&AiPaddle>,
    )>,
) {
    for (player, mut controller, ai) in query.iter_mut() {
        let mut translation = Vec2::ZERO;

        match (player, ai) {
            // Human player input handling
            (Player::P1, None) => {
                if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
                    translation.y += PADDLE_SPEED * time.delta_secs();
                }
                if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
                    translation.y -= PADDLE_SPEED * time.delta_secs();
                }
            }
            // AI player movement
            (Player::P2, Some(ai)) => {
                if ai.move_up {
                    translation.y += PADDLE_SPEED * time.delta_secs();
                }
                if ai.move_down {
                    translation.y -= PADDLE_SPEED * time.delta_secs();
                }
            }
            _ => {}
        }

        controller.translation = Some(translation);
    }
}

fn handle_paddle_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut paddle_query: Query<(Entity, &mut Transform, &mut PunchState), With<Player>>,
    ball_query: Query<Entity, With<Ball>>,
) {
    let Ok(ball_entity) = ball_query.get_single() else {
        return;
    };

    for collision_event in collision_events.read() {
        // Only handle actual contact events
        if let CollisionEvent::Started(e1, e2, flags) = collision_event {
            // Skip if neither entity is the ball
            if *e1 != ball_entity && *e2 != ball_entity {
                continue;
            }

            for (paddle_entity, mut transform, mut punch_state) in paddle_query.iter_mut() {
                if (paddle_entity == *e1 || paddle_entity == *e2) && !punch_state.is_punching {
                    punch_state.is_punching = true;
                    punch_state.timer.reset();

                    let punch_offset = if transform.translation.x < 0.0 {
                        0.15
                    } else {
                        -0.15
                    };
                    transform.translation.x += punch_offset;
                    break;
                }
            }
        }
    }
}

/// System to reset paddle position after punch
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

/// Spawns both player paddles: human P1 on left and AI P2 on right
fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create compound collider for the scoop shape
    let mut compound_collider = vec![];
    let mut all_vertices = vec![];

    // Generate segments for the scoop
    for i in 0..SEGMENTS {
        let vertices = generate_segment_vertices(i, SEGMENTS);

        // Add vertices to the complete mesh
        all_vertices.extend(vertices.iter().cloned());

        // Create collider for this segment
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
    for i in 0..SEGMENTS {
        let base = i as u32 * 4;
        // First triangle
        indices.extend_from_slice(&[base, base + 1, base + 2]);
        // Second triangle
        indices.extend_from_slice(&[base, base + 2, base + 3]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices_3d);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    // Spawn Player 1 (left paddle)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(material_handle.clone()),
        Transform::from_xyz(LEFT_PADDLE_X, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        InheritedVisibility::default(),
        RigidBody::KinematicPositionBased,
        Collider::compound(compound_collider.clone()),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        Player::P1,
        AdditionalMassProperties::Mass(0.1),
        PunchState {
            rest_x: LEFT_PADDLE_X,
            ..default()
        },
    ));

    // Spawn Player 2 (right paddle)
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform {
            translation: Vec3::new(RIGHT_PADDLE_X, 0.0, 0.0),
            rotation: Quat::from_rotation_z(std::f32::consts::PI), // Flip the paddle
            ..default()
        },
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        InheritedVisibility::default(),
        RigidBody::KinematicPositionBased,
        Collider::compound(compound_collider),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        Player::P2,
        AiPaddle::default(),
        AdditionalMassProperties::Mass(0.1),
        PunchState {
            rest_x: RIGHT_PADDLE_X,
            ..default()
        },
    ));
}

/// Plugin that manages all player-related systems
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_players).add_systems(
            Update,
            (
                ai_decision_making,
                paddle_movement,
                handle_paddle_collisions,
                update_paddle_punch,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
