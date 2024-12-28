//! Score Management Module
//!
//! Handles scoring mechanics and display for a table tennis-style game. Features include:
//! - Score tracking and persistence across game states
//! - Traditional table tennis scoring rules (first to 11, win by 2)
//! - Alternating serve patterns with deuce handling
//! - Score display UI with automatic updates
//! - Victory condition checking
//! - Ball spawning and serve mechanics

use crate::ball::{create_ball, Ball};
use crate::board::Wall;
use crate::GameState;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

// ----- Resources -----

/// Resource that tracks game scoring state and serve mechanics.
/// This persists across state changes to maintain game progress.
#[derive(Resource)]
pub struct Score {
    /// Player 1's current score
    pub p1: u32,
    /// Player 2's current score
    pub p2: u32,
    /// Indicates whether Player 1 is currently serving
    pub server_is_p1: bool,
    /// Tracks serves since last server switch
    serve_count: u32,
    /// Timer for delay between points
    serve_timer: Timer,
    /// Flag indicating a serve is pending
    pub should_serve: bool,
}

impl Score {
    /// Creates a new scoring state with initial values.
    /// Server is randomly chosen at start.
    fn new() -> Self {
        Self {
            p1: 0,
            p2: 0,
            server_is_p1: rand::thread_rng().gen_bool(0.5),
            serve_count: 0,
            serve_timer: Timer::from_seconds(0.75, TimerMode::Once),
            should_serve: false,
        }
    }

    /// Awards a point and handles serve rotation logic.
    ///
    /// Implements official table tennis serve rules:
    /// - Server changes every 2 points in normal play
    /// - Server changes every point during deuce (10-10 or higher)
    ///
    /// # Arguments
    /// * `p1_scored` - true if point goes to Player 1, false for Player 2
    fn add_point(&mut self, p1_scored: bool) {
        // Update appropriate player's score
        if p1_scored {
            self.p1 += 1;
        } else {
            self.p2 += 1;
        }

        self.serve_count += 1;

        // Check for deuce conditions (both players at 10+)
        let in_deuce = self.p1 >= 10 && self.p2 >= 10;
        let switch_threshold = if in_deuce { 1 } else { 2 };

        // Switch server if we've hit the threshold
        if self.serve_count >= switch_threshold {
            self.server_is_p1 = !self.server_is_p1;
            self.serve_count = 0;
        }
    }

    /// Checks if either player has won the game.
    ///
    /// Victory conditions (official table tennis rules):
    /// 1. Score must be 11 or higher
    /// 2. Must have a 2-point lead
    ///
    /// # Returns
    /// * `true` if either player has won
    /// * `false` if game should continue
    pub fn check_victory(&self) -> bool {
        if self.p1 >= 11 && self.p1 >= self.p2 + 2 {
            return true;
        }
        if self.p2 >= 11 && self.p2 >= self.p1 + 2 {
            return true;
        }
        false
    }

    /// Resets scoring state for a new game.
    ///
    /// This resets:
    /// - Both players' scores to 0
    /// - Serve count to 0
    /// - Randomly assigns initial server
    /// - Clears any pending serve state
    pub fn reset(&mut self) {
        self.p1 = 0;
        self.p2 = 0;
        self.server_is_p1 = rand::thread_rng().gen_bool(0.5);
        self.serve_count = 0;
        self.serve_timer.reset();
        self.should_serve = false;
    }
}

// ----- Components -----

/// Component to identify and differentiate score display UI elements.
#[derive(Component)]
struct ScoreText {
    kind: ScoreKind,
}

/// Types of score display UI elements.
enum ScoreKind {
    P1,   // Player 1's score display
    P2,   // Player 2's score display
    Root, // Container element
}

// ----- UI Creation and Management Systems -----

/// Creates the score display UI layout.
///
/// Layout structure:
/// - Root container (centered, fixed width)
///   - Player 1 score (left side)
///   - Player 2 score (right side)
///
/// # Arguments
/// * `commands` - Command buffer for entity creation
/// * `score` - Current score resource for initial values
fn setup_score_ui(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ScoreText {
                kind: ScoreKind::Root,
            },
        ))
        .with_children(|parent| {
            spawn_player_score(
                parent,
                score.p1,
                ScoreKind::P1,
                UiRect::right(Val::Px(20.0)),
            );
            spawn_player_score(parent, score.p2, ScoreKind::P2, UiRect::left(Val::Px(20.0)));
        });
}

/// Helper function to spawn individual player score displays.
///
/// # Arguments
/// * `parent` - Parent UI node to attach to
/// * `score` - Initial score value to display
/// * `kind` - Which player's score this represents
/// * `margin` - Margin settings for positioning
fn spawn_player_score(parent: &mut ChildBuilder, score: u32, kind: ScoreKind, margin: UiRect) {
    parent.spawn((
        Text::new(score.to_string()),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            margin,
            ..default()
        },
        ScoreText { kind },
    ));
}

/// Updates score display text to match current game state.
///
/// This system:
/// - Runs continuously during gameplay
/// - Updates only when text doesn't match current score
/// - Ensures consistency after state transitions
fn update_score_display(score: Res<Score>, mut query: Query<(&mut Text, &ScoreText)>) {
    for (mut text, score_text) in query.iter_mut() {
        let current_score = match score_text.kind {
            ScoreKind::P1 => score.p1,
            ScoreKind::P2 => score.p2,
            ScoreKind::Root => continue,
        };

        let score_text = current_score.to_string();
        if **text != score_text {
            **text = score_text;
        }
    }
}

/// Removes score display UI when leaving gameplay state.
fn cleanup_score_ui(mut commands: Commands, query: Query<Entity, With<ScoreText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ----- Gameplay Systems -----

/// Creates initial Score resource.
fn init_score(mut commands: Commands) {
    commands.insert_resource(Score::new());
}

/// Manages ball spawning for various game situations.
///
/// Spawns ball:
/// - At start of new game
/// - After resuming from pause
/// - After each point (with serve delay)
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

/// Implements serve delay mechanics between points.
///
/// This provides:
/// - Visual pause between points
/// - Time for players to prepare
/// - Consistent serve timing
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

/// Processes ball-wall collisions for scoring.
///
/// When ball hits scoring wall:
/// 1. Awards point to appropriate player
/// 2. Removes the ball
/// 3. Initiates serve sequence
fn handle_scoring(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut collision_events: EventReader<CollisionEvent>,
    ball_query: Query<Entity, With<Ball>>,
    wall_query: Query<(Entity, &Wall)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Find colliding entities
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

/// Monitors for victory conditions during gameplay.
///
/// When victory detected:
/// 1. Removes the ball to prevent further scoring
/// 2. Transitions to game over state
fn check_victory(
    score: Res<Score>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    ball_query: Query<Entity, With<Ball>>,
) {
    if score.check_victory() {
        for entity in ball_query.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::GameOver);
    }
}

// ----- Plugin Setup -----

/// Plugin that manages all scoring functionality.
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resource initialization
            .add_systems(Startup, init_score)
            // UI management
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_score_ui, update_score_display),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_score_ui)
            .add_systems(OnEnter(GameState::Playing), on_resume)
            // Score display updates
            .add_systems(
                Update,
                update_score_display.run_if(in_state(GameState::Playing)),
            )
            // Gameplay systems
            .add_systems(
                Update,
                (handle_scoring, handle_serve_delay, check_victory)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
