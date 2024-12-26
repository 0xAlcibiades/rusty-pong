use crate::ball::{create_ball, Ball};
use crate::board::Wall;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

/// Tracks game score and serving state, implementing standard table tennis rules:
/// - Server alternates every 2 points during normal play
/// - During deuce (10-10 or higher), server alternates every point
/// - Initial server is randomly chosen at game start
/// - 1-second delay between points
#[derive(Resource)]
pub struct Score {
    /// Current score for Player 1 (left side)
    pub p1: u32,
    /// Current score for Player 2 (right side)
    pub p2: u32,
    /// True if Player 1 is currently serving, false for Player 2
    pub server_is_p1: bool,
    /// Number of points played since last server change
    serve_count: u32,
    /// Timer to control delay between points
    serve_timer: Timer,
    /// Flag indicating when a new serve should occur after delay
    should_serve: bool,
}

impl Score {
    /// Creates a new score tracker with scores at 0 and random initial server
    fn new() -> Self {
        Self {
            p1: 0,
            p2: 0,
            server_is_p1: rand::thread_rng().gen_bool(0.5),
            serve_count: 0,
            // A 300ms delay between points
            serve_timer: Timer::from_seconds(0.3, TimerMode::Once),
            should_serve: false,
        }
    }

    /// Increments score and handles serve switching according to table tennis rules
    fn add_point(&mut self, p1_scored: bool) {
        if p1_scored {
            self.p1 += 1;
        } else {
            self.p2 += 1;
        }

        self.serve_count += 1;

        let in_deuce = self.p1 >= 10 && self.p2 >= 10;
        let switch_threshold = if in_deuce { 1 } else { 2 };

        if self.serve_count >= switch_threshold {
            self.server_is_p1 = !self.server_is_p1;
            self.serve_count = 0;
        }
    }
}

/// Component to identify score display text elements
#[derive(Component)]
enum ScoreText {
    /// Left side score (Player 1)
    P1,
    /// Right side score (Player 2)
    P2,
}

/// Sets up the score display UI at the top center of the screen
///
/// Creates:
/// - Root node spanning full width for centering
/// - Two text elements displaying scores
/// - Proper spacing between scores
///
/// The scores are positioned absolutely at the top of the screen
/// and centered horizontally using flexbox layout.
fn setup_score_ui(mut commands: Commands) {
    // Initialize the score tracking resource
    commands.insert_resource(Score::new());

    // Root node - provides centering and layout for score elements
    commands
        .spawn(Node {
            // Take full width for centering
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            // Position at top of screen with some padding
            top: Val::Px(20.0),
            // Center children horizontally
            justify_content: JustifyContent::Center,
            // Use flexbox layout
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            // Player 1 score (left side)
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::right(Val::Px(20.0)), // Space between scores
                    ..default()
                },
                ScoreText::P1,
            ));

            // Player 2 score (right side)
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::left(Val::Px(20.0)), // Space between scores
                    ..default()
                },
                ScoreText::P2,
            ));
        });
}

/// System that handles serving delay and ball spawning
///
/// After a point is scored, waits for the delay timer before spawning
/// a new ball. The ball's direction is based on which player is serving.
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

/// System that detects ball collisions with walls and updates score
///
/// When the ball hits a scoring wall:
/// - Updates the score based on which wall was hit
/// - Removes the ball from play
/// - Triggers the serve delay timer
fn handle_scoring(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut collision_events: EventReader<CollisionEvent>,
    ball_query: Query<Entity, With<Ball>>,
    wall_query: Query<(Entity, &Wall)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            let ball_entity = ball_query.iter().find(|e| *e == *e1 || *e == *e2);
            let wall = wall_query
                .iter()
                .find(|(e, _)| *e == *e1 || *e == *e2)
                .map(|(_, w)| w);

            if let (Some(ball_entity), Some(wall)) = (ball_entity, wall) {
                match wall {
                    Wall::Left => {
                        // P2 scores
                        score.add_point(false);
                        commands.entity(ball_entity).despawn();
                        score.should_serve = true;
                    }
                    Wall::Right => {
                        // P1 scores
                        score.add_point(true);
                        commands.entity(ball_entity).despawn();
                        score.should_serve = true;
                    }
                    _ => {} // Ignore top/bottom wall collisions
                }
            }
        }
    }
}

/// Updates the score display whenever the score resource changes
///
/// Monitors the score resource and updates both player score displays
/// to show their current points. Only runs when the score actually changes.
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

/// Plugin that manages all scoring functionality for the game
///
/// Provides:
/// - Score tracking and serve management
/// - Score display UI
/// - Collision detection for scoring
/// - Display updates
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_ui).add_systems(
            Update,
            (handle_scoring, update_score_display, handle_serve_delay),
        );
    }
}
