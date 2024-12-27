//! Endgame Screen Module
//!
//! This module handles the game's victory screen, including:
//! - Victory/Defeat message display
//! - Final score display
//! - Prompt for starting a new game
//! - Game state reset functionality

use crate::score::Score;
use crate::GameState;
use bevy::prelude::*;

/// Plugin that manages the victory screen functionality
pub struct EndgamePlugin;

/// Marker component for identifying victory screen UI elements
#[derive(Component)]
struct EndgameScreen;

impl Plugin for EndgamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Create victory screen when entering GameOver state
            .add_systems(OnEnter(GameState::GameOver), spawn_endgame_screen)
            // Handle space bar input while in GameOver state
            .add_systems(
                Update,
                handle_endgame_input.run_if(in_state(GameState::GameOver)),
            )
            // Clean up victory screen when leaving GameOver state
            .add_systems(OnExit(GameState::GameOver), despawn_endgame_screen);
    }
}

/// Spawns the victory screen UI elements
fn spawn_endgame_screen(mut commands: Commands, score: Res<Score>) {
    let (message, color) = if score.p1 > score.p2 {
        ("Victory!", Color::srgba(26.0, 228.0, 61.0, 1.0)) // Complementary green
    } else {
        ("Defeat!", Color::srgba(228.0, 61.0, 26.0, 1.0)) // Rust orange
    };

    commands
        .spawn((
            EndgameScreen,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Victory/Defeat message
            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Final score
            parent.spawn((
                Text::new(format!("Final Score: {} - {}", score.p1, score.p2)),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Play again prompt
            parent.spawn((
                Text::new("Press SPACE to play again"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node::default(),
            ));
        });
}

/// Handles keyboard input on the victory screen
fn handle_endgame_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Reset score and start new game
        score.reset();
        next_state.set(GameState::Playing);
    }
}

/// Cleans up victory screen entities
fn despawn_endgame_screen(mut commands: Commands, screen: Query<Entity, With<EndgameScreen>>) {
    for entity in screen.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
