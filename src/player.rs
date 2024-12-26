use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Constants defining the player paddles' properties.
/// These values are in world units, where the game board is 16 units wide
/// and 10 units high, with (0,0) at the center.
const PADDLE_WIDTH: f32 = 0.5;
const PADDLE_HEIGHT: f32 = 2.0;

/// Movement speed in world units per second.
/// Since our board is 10 units high, a speed of 5.0 means we can traverse
/// half the board height in one second.
const PADDLE_SPEED: f32 = 5.0;

/// Starting X positions for the paddles, positioned just inside the board walls.
/// The board is 16 units wide with 0.1 unit thick walls.
/// Calculation for right paddle: board_width/2 - wall_thickness - paddle_width/2
/// Left paddle uses the negative of this value.
const LEFT_PADDLE_X: f32 = -7.65; // -(8.0 - 0.1 - (0.5/2))
const RIGHT_PADDLE_X: f32 = 7.65; // 8.0 - 0.1 - (0.5/2)

/// Component that identifies which player a paddle belongs to.
/// Currently only Player 1 (left side) is controllable.
#[derive(Component)]
pub enum Player {
    /// Left side player, controlled by W/S keys
    P1,
    /// Right side player, will be controlled later
    P2,
}

/// System that handles player movement based on keyboard input.
///
/// Currently only moves Player 1's paddle using:
/// - W key: Move up
/// - S key: Move down
///
/// Movement is handled through a KinematicCharacterController, which automatically
/// manages collisions with the board walls.
///
/// # Arguments
/// * `input` - Resource for keyboard input state
/// * `time` - Resource for frame timing information
/// * `query` - Query for Player component and KinematicCharacterController
fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Player, &mut KinematicCharacterController)>,
) {
    // Iterate through paddles (currently only Player 1 will be moved)
    for (player, mut controller) in query.iter_mut() {
        // Only move Player 1's paddle
        if let Player::P1 = player {
            let mut translation = Vec2::ZERO;

            // Calculate vertical movement based on input
            if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
                translation.y += PADDLE_SPEED * time.delta_secs();
            }
            if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
                translation.y -= PADDLE_SPEED * time.delta_secs();
            }

            // Apply movement through the character controller
            controller.translation = Some(translation);
        }
    }
}

/// Spawns both player paddles at their starting positions.
///
/// Creates two paddles:
/// - Player 1 on the left side
/// - Player 2 on the right side
///
/// Each paddle is created with:
/// - Visual components (Sprite, Transform, etc.)
/// - Physics components (RigidBody, Collider, KinematicCharacterController)
/// - Player component to identify which paddle it is
fn spawn_players(mut commands: Commands) {
    // Spawn Player 1 (left paddle)
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
        RigidBody::KinematicPositionBased, // Uses direct position control
        Collider::cuboid(PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0), // Box collider
        KinematicCharacterController::default(), // Handles movement and collisions
        // Gameplay component
        Player::P1, // Marks this as Player 1's paddle
    ));

    // Spawn Player 2 (right paddle)
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
        // Gameplay component
        Player::P2, // Marks this as Player 2's paddle
    ));
}

/// Plugin that manages all player-related functionality.
///
/// This plugin is responsible for:
/// - Spawning both player paddles
/// - Handling player movement input
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_players) // Spawn paddles when game starts
            .add_systems(Update, player_movement); // Handle movement each frame
    }
}
