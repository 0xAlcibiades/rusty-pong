//! Score Management Module
//!
//! This module handles all scoring-related functionality, including:
//! - Score tracking and display
//! - Serve mechanics and timing
//! - Ball spawning on points
//! - Score UI rendering
//! - Victory conditions and game reset
//!
//! The scoring system follows traditional ping-pong rules with
//! serve switching and deuce handling.

use crate::ball::{create_ball, Ball};
use crate::board::Wall;
use crate::GameState;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

/// Resource that tracks the game's scoring state
#[derive(Resource)]
pub struct Score {
    pub p1: u32,            // Player 1's score
    pub p2: u32,            // Player 2's score
    pub server_is_p1: bool, // Whether P1 is currently serving
    serve_count: u32,       // Number of serves since last switch
    serve_timer: Timer,     // Delay between point and next serve
    pub should_serve: bool, // Whether we're waiting to serve
}

impl Score {
    /// Creates a new score state with initial values
    fn new() -> Self {
        Self {
            p1: 0,
            p2: 0,
            // Randomly choose initial server
            server_is_p1: rand::thread_rng().gen_bool(0.5),
            serve_count: 0,
            // 0.75 second delay before serving
            serve_timer: Timer::from_seconds(0.75, TimerMode::Once),
            should_serve: false,
        }
    }

    /// Adds a point and handles serve switching logic
    ///
    /// # Arguments
    /// * `p1_scored` - Whether player 1 scored the point
    ///
    /// # Serve Switching Rules
    /// - Normal play: Switch server every 2 points
    /// - Deuce (10-10 or higher): Switch every point
    fn add_point(&mut self, p1_scored: bool) {
        // Update score
        if p1_scored {
            self.p1 += 1;
        } else {
            self.p2 += 1;
        }

        self.serve_count += 1;

        // Check for deuce conditions
        let in_deuce = self.p1 >= 10 && self.p2 >= 10;
        let switch_threshold = if in_deuce { 1 } else { 2 };

        // Switch server if threshold reached
        if self.serve_count >= switch_threshold {
            self.server_is_p1 = !self.server_is_p1;
            self.serve_count = 0;
        }
    }

    /// Checks if the game has reached a victory condition
    ///
    /// Victory is achieved when:
    /// - A player reaches 11 or more points AND
    /// - They have a lead of at least 2 points
    ///
    /// # Returns
    /// `true` if either player has won, `false` if the game should continue
    pub fn check_victory(&self) -> bool {
        if self.p1 >= 11 && self.p1 >= self.p2 + 2 {
            return true;
        }
        if self.p2 >= 11 && self.p2 >= self.p1 + 2 {
            return true;
        }
        false
    }

    /// Resets the game state to initial values for a new game
    ///
    /// This:
    /// - Clears both players' scores
    /// - Randomly assigns a new server
    /// - Resets serve count and timer
    /// - Clears any pending serve flags
    pub fn reset(&mut self) {
        self.p1 = 0;
        self.p2 = 0;
        self.server_is_p1 = rand::thread_rng().gen_bool(0.5);
        self.serve_count = 0;
        self.serve_timer.reset();
        self.should_serve = false;
    }
}

/// Component to identify score display text entities
#[derive(Component)]
enum ScoreText {
    P1, // Player 1's score text
    P2, // Player 2's score text
}

/// Sets up the score display UI
///
/// Creates:
/// - Score resource initialization
/// - Centered score display container
/// - Player score text elements
fn setup_score_ui(mut commands: Commands) {
    // Initialize score resource
    commands.insert_resource(Score::new());

    // Create UI container
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            // Player 1 score text
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
                ScoreText::P1,
            ));

            // Player 2 score text
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::left(Val::Px(20.0)),
                    ..default()
                },
                ScoreText::P2,
            ));
        });
}

/// Spawns a new ball when entering Playing state or resuming from pause
///
/// Only spawns if:
/// - No ball currently exists
/// - Not currently in serve delay
fn on_resume(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    score: Res<Score>,
    ball_query: Query<Entity, With<Ball>>,
) {
    if ball_query.is_empty() && !score.should_serve {
        create_ball(
            &mut commands,
            &mut meshes,
            &mut materials,
            score.server_is_p1,
        );
    }
}

/// Checks for victory conditions and transitions to game over state
///
/// When victory is achieved:
/// 1. Despawns the ball to prevent further scoring
/// 2. Transitions to the GameOver state
fn check_victory(
    score: Res<Score>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    ball_query: Query<Entity, With<Ball>>,
) {
    if score.check_victory() {
        // Despawn the ball to prevent further scoring
        for entity in ball_query.iter() {
            commands.entity(entity).despawn();
        }
        // Transition to game over state
        next_state.set(GameState::GameOver);
    }
}

/// Handles the delay between scoring and serving
///
/// Provides a brief pause after points to:
/// - Let players see what happened
/// - Prepare for the next serve
/// - Reset game state
fn handle_serve_delay(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if score.should_serve {
        score.serve_timer.tick(time.delta());

        if score.serve_timer.just_finished() {
            create_ball(
                &mut commands,
                &mut meshes,
                &mut materials,
                score.server_is_p1,
            );
            score.should_serve = false;
            score.serve_timer.reset();
        }
    }
}

/// Handles ball collisions with scoring walls
///
/// When ball hits left/right walls:
/// 1. Updates appropriate player's score
/// 2. Despawns the ball
/// 3. Triggers serve delay
fn handle_scoring(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut collision_events: EventReader<CollisionEvent>,
    ball_query: Query<Entity, With<Ball>>,
    wall_query: Query<(Entity, &Wall)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Find the ball and wall entities involved
            let ball_entity = ball_query.iter().find(|e| *e == *e1 || *e == *e2);
            let wall = wall_query
                .iter()
                .find(|(e, _)| *e == *e1 || *e == *e2)
                .map(|(_, w)| w);

            if let (Some(ball_entity), Some(wall)) = (ball_entity, wall) {
                match wall {
                    Wall::Left => {
                        score.add_point(false); // P2 scores
                        commands.entity(ball_entity).despawn();
                        score.should_serve = true;
                    }
                    Wall::Right => {
                        score.add_point(true); // P1 scores
                        commands.entity(ball_entity).despawn();
                        score.should_serve = true;
                    }
                    _ => {} // Top/Bottom walls don't affect score
                }
            }
        }
    }
}

/// Updates the score display text when scores change
fn update_score_display(score: Res<Score>, mut query: Query<(&mut Text, &ScoreText)>) {
    if score.is_changed() {
        for (mut text, score_type) in query.iter_mut() {
            match score_type {
                ScoreText::P1 => {
                    **text = score.p1.to_string();
                }
                ScoreText::P2 => {
                    **text = score.p2.to_string();
                }
            }
        }
    }
}

/// Plugin that manages scoring functionality
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize UI at startup
            .add_systems(Startup, setup_score_ui)
            // Handle ball spawning when entering Playing state
            .add_systems(OnEnter(GameState::Playing), on_resume)
            // Update score display whenever scores change
            .add_systems(Update, update_score_display)
            // Handle scoring, serving, and victory checking during gameplay
            .add_systems(
                Update,
                (handle_scoring, handle_serve_delay, check_victory)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
