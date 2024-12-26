use crate::ball::Ball;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Constants defining the player paddles' properties
const PADDLE_WIDTH: f32 = 0.5;
const PADDLE_HEIGHT: f32 = 2.0;
const PADDLE_SPEED: f32 = 5.0;
const MAX_BOUNCE_ANGLE: f32 = 1.0;
const BALL_SPEED: f32 = 10.0;
const LEFT_PADDLE_X: f32 = -7.65;
const RIGHT_PADDLE_X: f32 = 7.65;

/// Update rate for AI decisions
const AI_UPDATE_RATE: f32 = 0.3; // 3.33 times per second, like a human

#[derive(Component)]
pub enum Player {
    P1,
    P2,
}

/// Component for AI-controlled paddles that simulates input
#[derive(Component)]
struct AiPaddle {
    update_timer: Timer,
    move_up: bool,   // Simulates W key
    move_down: bool, // Simulates S key
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

/// Predicts where the ball will intersect the paddle's x-position
fn predict_intersection(ball_pos: Vec2, ball_vel: Vec2, paddle_x: f32) -> Option<f32> {
    // Only predict if ball is moving toward the paddle
    if (paddle_x > ball_pos.x && ball_vel.x > 0.0) || (paddle_x < ball_pos.x && ball_vel.x < 0.0) {
        let time = (paddle_x - ball_pos.x) / ball_vel.x;
        let y = ball_pos.y + (ball_vel.y * time);
        Some(y)
    } else {
        None
    }
}

/// System that makes AI decisions about which "keys" to press
fn ai_decision_making(
    time: Res<Time>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut ai_query: Query<(&Transform, &mut AiPaddle)>,
) {
    for (paddle_transform, mut ai) in ai_query.iter_mut() {
        // Update AI decisions periodically
        if ai.update_timer.tick(time.delta()).just_finished() {
            if let Ok((ball_transform, ball_velocity)) = ball_query.get_single() {
                if let Some(predicted_y) = predict_intersection(
                    ball_transform.translation.truncate(),
                    ball_velocity.linvel,
                    RIGHT_PADDLE_X,
                ) {
                    // Decide whether to move up or down based on prediction
                    let diff = predicted_y - paddle_transform.translation.y;

                    // Clear previous movement
                    ai.move_up = false;
                    ai.move_down = false;

                    // Choose direction based on target position
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

/// Single movement system that handles both player and AI input
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
            // Player 1: Use keyboard input
            (Player::P1, None) => {
                if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
                    translation.y += PADDLE_SPEED * time.delta_secs();
                }
                if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
                    translation.y -= PADDLE_SPEED * time.delta_secs();
                }
            }
            // AI Player: Use simulated input
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

fn handle_paddle_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    paddle_query: Query<(&Transform, &Player)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
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

fn handle_bounce(
    ball_transform: &Transform,
    ball_vel: &mut Velocity,
    paddle_transform: &Transform,
) {
    let relative_hit =
        (ball_transform.translation.y - paddle_transform.translation.y) / (PADDLE_HEIGHT / 2.0);
    let bounce_angle = relative_hit * MAX_BOUNCE_ANGLE;
    let direction = if ball_vel.linvel.x > 0.0 { 1.0 } else { -1.0 };

    ball_vel.linvel = Vec2::new(
        -direction * BALL_SPEED * bounce_angle.cos(),
        BALL_SPEED * bounce_angle.sin(),
    );
}

fn spawn_players(mut commands: Commands) {
    // Player 1 (left paddle)
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(LEFT_PADDLE_X, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        RigidBody::KinematicPositionBased,
        Collider::cuboid(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        Player::P1,
    ));

    // Player 2 (right paddle) with AI
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(RIGHT_PADDLE_X, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        RigidBody::KinematicPositionBased,
        Collider::cuboid(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0),
        KinematicCharacterController::default(),
        ActiveEvents::COLLISION_EVENTS,
        Player::P2,
        AiPaddle::default(),
    ));
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_players).add_systems(
            Update,
            (ai_decision_making, paddle_movement, handle_paddle_collision),
        );
    }
}
