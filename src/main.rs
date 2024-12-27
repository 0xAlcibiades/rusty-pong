//! Rusty Pong - A Pong clone built with Bevy
//!
//! This is the main entry point for the game. It sets up the core game systems,
//! manages the game state, and coordinates all the various plugins that make up
//! the game's functionality.
//!
//! Game Flow:
//! 1. Starts at splash screen (Splash state)
//! 2. Press space to begin gameplay (Playing state)
//! 3. Game can be paused at any time (Paused state)
//! 4. When a player wins, shows victory/defeat screen (GameOver state)
//! 5. From victory/defeat, can start a new game (returns to Playing state)

use bevy::app::{App, PluginGroup};
use bevy::prelude::Update;
use bevy::prelude::{AppExtStates, States};
use bevy::DefaultPlugins;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};

// Import all our game's plugins and modules
use crate::audio::MusicPlugin;
use crate::ball::BallPlugin;
use crate::board::BoardPlugin;
use crate::camera::CameraPlugin;
use crate::endgame::EndgamePlugin;
use crate::pause::{handle_pause, PausePlugin};
use crate::player::PlayerPlugin;
use crate::score::ScorePlugin;
use crate::splash::SplashPlugin;
use crate::window::default_window_plugin;

// Declare all our game's modules
mod audio; // Handles background music and sound effects
mod ball; // Ball physics and behavior
mod board; // Game board and walls
mod camera; // Camera setup and configuration
mod endgame;
mod pause; // Pause menu and state management
mod player; // Player paddles and controls
mod score; // Score tracking and display
mod splash; // Splash screen
mod window; // Window configuration // Victory/Defeat screen

/// Represents the different states the game can be in.
/// The game's behavior and active systems change based on the current state.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Splash, // Initial splash screen, entry point of the game
    Playing,  // Active gameplay where players compete
    Paused,   // Game is temporarily paused, showing pause menu
    GameOver, // Game has ended with a winner, showing victory/defeat screen
}

/// Groups all gameplay-related plugins together for better organization
/// and easier initialization.
///
/// Plugins are added in a specific order to ensure proper initialization:
/// 1. Board setup (walls and background)
/// 2. Player systems (paddles and controls)
/// 3. Camera configuration
/// 4. Ball physics and behavior
/// 5. Scoring system
/// 6. Audio features
struct GamePlayPlugins;

impl PluginGroup for GamePlayPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            // Add core gameplay plugins in a logical order
            .add(BoardPlugin) // First setup the game board
            .add(PlayerPlugin) // Then add players
            .add(CameraPlugin) // Setup the camera to view the game
            .add(BallPlugin) // Add the ball
            .add(ScorePlugin) // Add scoring system
            .add(MusicPlugin) // Finally add audio
    }
}

/// The main entry point for the game.
/// Sets up the Bevy app with all required plugins and systems.
fn main() {
    App::new()
        .add_plugins((
            // Setup default Bevy plugins with our custom window configuration
            DefaultPlugins.set(default_window_plugin()),
            // Add physics engine with scaling configured for our coordinate system
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // Add our game-specific plugins in order of state flow
            SplashPlugin,    // Initial splash screen
            PausePlugin,     // Pause functionality
            EndgamePlugin,   // Victory/defeat screen
            GamePlayPlugins, // Core gameplay systems
        ))
        // Initialize the game state system
        .init_state::<GameState>()
        // Add the pause handling system to run during updates
        .add_systems(Update, handle_pause)
        // Start the game
        .run();
}
