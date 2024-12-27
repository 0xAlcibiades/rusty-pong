//! Pause System Module
//!
//! This module handles the game's pause functionality, including:
//! - Pause menu UI creation and cleanup
//! - State transitions between Playing and Paused states
//! - Space key input handling for pause toggling
//!
//! The pause system uses Bevy's UI system for menu rendering and
//! state system for game state management.

use crate::GameState;
use bevy::prelude::*;

/// Marker component for identifying pause menu entities.
/// Used for querying and cleanup when the pause state exits.
#[derive(Component)]
struct PauseMenu;

/// Plugin that manages pause functionality.
///
/// Responsible for:
/// - Spawning the pause menu when entering paused state
/// - Cleaning up the menu when exiting paused state
pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            // Spawn pause menu when entering paused state
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            // Cleanup menu when exiting paused state
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu);
    }
}

/// Spawns the pause menu UI when the game enters the paused state.
///
/// Creates a full-screen, semi-transparent overlay with:
/// - Centered "PAUSED" text in large font
/// - "Press SPACE to continue" prompt below
///
/// The menu uses flexbox layout for:
/// - Vertical stacking of elements
/// - Center alignment both horizontally and vertically
/// - Full screen coverage
fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            // Mark as pause menu for later cleanup
            PauseMenu,
            // Root node configuration
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
            // Semi-transparent black overlay
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // "PAUSED" text
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 80.0, // Large, prominent text
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    // Add space below the title
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // "Press SPACE to continue" prompt
            parent.spawn((
                Text::new("Press SPACE to continue"),
                TextFont {
                    font_size: 40.0, // Smaller than title
                    ..default()
                },
                TextColor(Color::WHITE),
                Node::default(),
            ));
        });
}

/// Cleans up the pause menu when exiting the paused state.
///
/// Queries for all entities with the PauseMenu component and
/// recursively despawns them and their children.
fn despawn_pause_menu(mut commands: Commands, pause_menu: Query<Entity, With<PauseMenu>>) {
    for entity in pause_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// System that handles pausing and unpausing the game when space is pressed.
/// Only toggles between Playing and Paused states, ignoring other states
/// (like the splash screen).
///
/// # State Transitions
/// - Playing → Paused: When space pressed during gameplay
/// - Paused → Playing: When space pressed while paused
/// - Other states: No effect
pub(crate) fn handle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,  // Keyboard input resource
    current_state: Res<State<GameState>>, // Current game state
    mut next_state: ResMut<NextState<GameState>>, // For changing game state
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => (), // Do nothing in other states (like Splash)
        }
    }
}
