//! Splash Screen Module
//!
//! This module handles the game's splash screen, including:
//! - Initial screen display and layout
//! - Title and prompt rendering
//! - Input handling for game start
//! - Transition to gameplay
//!
//! The splash screen serves as the initial game state and
//! provides a clean entry point to the game.

use crate::GameState;
use bevy::prelude::*;

/// Plugin that manages the splash screen functionality.
///
/// This plugin coordinates:
/// - Splash screen creation on game start
/// - Input handling for transitioning to gameplay
/// - Cleanup when transitioning to game
pub struct SplashPlugin;

/// Marker component for identifying splash screen UI elements.
/// Used for querying and cleanup when transitioning to gameplay.
#[derive(Component)]
struct SplashScreen;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app
            // Create splash screen when entering Splash state
            .add_systems(OnEnter(GameState::Splash), spawn_splash_screen)
            // Handle space bar input while in Splash state
            .add_systems(
                Update,
                handle_splash_input.run_if(in_state(GameState::Splash)),
            )
            // Clean up splash screen when leaving Splash state
            .add_systems(OnExit(GameState::Splash), despawn_splash_screen);
    }
}

/// Spawns the splash screen UI elements.
///
/// Creates a full-screen layout containing:
/// - Game title ("Rusty Pong")
/// - Start prompt ("Press SPACE to start")
///
/// The layout uses flexbox for:
/// - Vertical stacking of elements
/// - Center alignment both horizontally and vertically
/// - Full screen coverage with black background
fn spawn_splash_screen(mut commands: Commands) {
    // Create root container node
    commands
        .spawn((
            // Mark as splash screen for later cleanup
            SplashScreen,
            // Root node layout configuration
            Node {
                // Use flexbox layout
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,

                // Take up full screen
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // Black background
            BackgroundColor(Color::BLACK),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new("Rusty Pong"),
                TextFont {
                    font_size: 80.0, // Large, prominent title
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    // Add space below title
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Start game prompt
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont {
                    font_size: 40.0, // Smaller than title
                    ..default()
                },
                TextColor(Color::WHITE),
                Node::default(),
            ));
        });
}

/// Handles keyboard input on the splash screen.
///
/// Watches for space bar press and transitions to
/// the Playing state when detected.
fn handle_splash_input(
    keyboard: Res<ButtonInput<KeyCode>>, // Keyboard input resource
    mut next_state: ResMut<NextState<GameState>>, // For state transitions
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing); // Start the game
    }
}

/// Cleans up splash screen entities when transitioning to gameplay.
///
/// Queries for all entities marked with the SplashScreen component
/// and recursively despawns them and their children.
fn despawn_splash_screen(mut commands: Commands, splash_screen: Query<Entity, With<SplashScreen>>) {
    for entity in splash_screen.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
