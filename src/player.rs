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
use bevy_rapier2d::prelude::*;

/// Constants defining the player paddles' physical and gameplay properties
const PADDLE_WIDTH: f32 = 0.5; // Width in world units
const PADDLE_HEIGHT: f32 = 2.0; // Height in world units
const PADDLE_SPEED: f32 = 5.0; // Movement speed in units per second
const MAX_BOUNCE_ANGLE: f32 = 1.0; // Maximum angle ball can bounce (radians)
const LEFT_PADDLE_X: f32 = -7.65; // X position of left paddle
const RIGHT_PADDLE_X: f32 = 7.65; // X position of right paddle

/// Update rate for AI decisions to simulate human-like reaction time
const AI_UPDATE_RATE: f32 = 0.3; // Updates 3.33 times per second

/// Component that identifies which player a paddle belongs to
#[derive(Component)]
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

impl Default for AiPaddle {
    fn default() -> Self {
        Self {
            update_timer: Timer::from_seconds(AI_UPDATE_RATE, TimerMode::Repeating),
            move_up: false,
            move_down: false,
        }
    }
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
        // Update AI decisions at fixed intervals
        if ai.update_timer.tick(time.delta()).just_finished() {
            if let Ok((ball_transform, ball_velocity)) = ball_query.get_single() {
                if let Some(predicted_y) = predict_intersection(
                    ball_transform.translation.truncate(),
                    ball_velocity.linvel,
                    RIGHT_PADDLE_X,
                ) {
                    // Calculate difference between current and target position
                    let diff = predicted_y - paddle_transform.translation.y;

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
///
/// For human players:
/// - W/Up Arrow: Move up
/// - S/Down Arrow: Move down
///
/// For AI:
/// - Uses simulated input from AI decision making system
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

/// Handles ball-paddle collision events and calculates bounce trajectories
fn handle_paddle_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    paddle_query: Query<(&Transform, &Player)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Check both collision entity orders
            if let Ok((ball_transform, mut ball_vel)) = ball_query.get_mut(*e1) {
                if let Ok((paddle_transform, _)) = paddle_query.get(*e2) {
                    handle_bounce(ball_transform, &mut ball_vel, paddle_transform);
                }
            } else if let Ok((ball_transform, mut ball_vel)) = ball_query.get_mut(*e2) {
                if let Ok((paddle_transform, _)) = paddle_query.get(*e1) {
                    handle_bounce(ball_transform, &mut ball_vel, paddle_transform);
                }
            }
        }
    }
}

/// Calculates and applies ball bounce physics based on paddle hit position
///
/// The bounce angle is determined by where the ball hits the paddle:
/// - Center: Straight bounce
/// - Top: Upward angle
/// - Bottom: Downward angle
fn handle_bounce(
    ball_transform: &Transform,
    ball_vel: &mut Velocity,
    paddle_transform: &Transform,
) {
    // Calculate relative hit position (-1 to 1)
    let relative_hit =
        (ball_transform.translation.y - paddle_transform.translation.y) / (PADDLE_HEIGHT / 2.0);

    // Calculate bounce angle based on hit position
    let bounce_angle = relative_hit * MAX_BOUNCE_ANGLE;

    // Preserve the ball's speed magnitude
    let current_speed = ball_vel.linvel.length();
    let direction = if ball_vel.linvel.x > 0.0 { 1.0 } else { -1.0 };

    // Apply new velocity vector while maintaining speed
    ball_vel.linvel = Vec2::new(
        -direction * current_speed * bounce_angle.cos(),
        current_speed * bounce_angle.sin(),
    );
}

/// Spawns both player paddles: human P1 on left and AI P2 on right
fn spawn_players(mut commands: Commands) {
    // Player 1 (left paddle, human controlled)
    commands.spawn((
        // Visual components
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(LEFT_PADDLE_X, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        // Physics components
        RigidBody::KinematicPositionBased,
        Collider::cuboid(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        // Player identification
        Player::P1,
    ));

    // Player 2 (right paddle, AI controlled)
    commands.spawn((
        // Visual components
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(RIGHT_PADDLE_X, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        // Physics components
        RigidBody::KinematicPositionBased,
        Collider::cuboid(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        // Player and AI components
        Player::P2,
        AiPaddle::default(),
    ));
}

/// Plugin that manages all player-related systems
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Spawn players at startup
            .add_systems(Startup, spawn_players)
            // Add gameplay systems that run only in Playing state
            .add_systems(
                Update,
                (ai_decision_making, paddle_movement, handle_paddle_collision)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
