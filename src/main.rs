use crate::ball::BallPlugin;
use crate::board::BoardPlugin;
use crate::camera::CameraPlugin;
use crate::player::PlayerPlugin;
use crate::window::default_window_plugin;
use bevy::app::{App, PluginGroup};
use bevy::DefaultPlugins;
use bevy_rapier2d::prelude::*;

/// Module containing game board setup and wall creation
mod board;
/// Module for camera configuration and setup
mod camera;
/// Module for player paddle creation and movement
mod player;
/// Module for window configuration, especially web-specific settings
mod window;

/// Module for ball entity creation and movement
mod ball;

/// Entry point for the game.
///
/// Sets up the Bevy game engine with all necessary plugins:
/// - Default Bevy plugins (rendering, windowing, input, etc.)
/// - Physics system (Rapier2D)
/// - Game-specific plugins (board, players, camera)
///
/// # Physics Configuration
/// The physics system is configured with a scale of 100 pixels per meter,
/// which means:
/// - 1 unit in our game logic = 100 pixels on screen
/// - This helps maintain sensible numbers for physics calculations
///
/// # Plugin Order
/// The plugins are added in a specific order:
/// 1. DefaultPlugins (core Bevy functionality)
/// 2. RapierPhysicsPlugin (physics system)
/// 3. BoardPlugin (game board setup)
/// 4. PlayerPlugin (player entities and controls)
/// 5. CameraPlugin (game camera configuration)
fn main() {
    App::new()
        .add_plugins((
            // Core Bevy plugins with custom window configuration
            DefaultPlugins.set(
                // Configure window for web browser deployment
                default_window_plugin(),
            ),
            // Physics plugin configuration
            // NoUserData indicates we're not adding custom data to physics objects
            // pixels_per_meter sets the conversion ratio between physics and screen space
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // Game-specific plugins
            BoardPlugin,  // Sets up the game board and walls
            PlayerPlugin, // Creates and manages the player paddles
            CameraPlugin, // Configures the game camera
            BallPlugin,   // Adds the ball entity and movement
        ))
        .run();
}
