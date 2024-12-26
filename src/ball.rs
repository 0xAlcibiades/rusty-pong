use crate::board::Wall;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

/// Ball diameter in world units.
const BALL_SIZE: f32 = 0.3;
/// Initial ball velocity in world units per second.
const BALL_SPEED: f32 = 10.0;
/// Threshold for velocity correction.
const SPEED_TOLERANCE: f32 = 0.1;

/// Component marking an entity as the game ball.
#[derive(Component)]
pub struct Ball;

/// Events triggered when a player scores.
#[derive(Event)]
pub enum ScoreEvent {
    P1Scored,
    P2Scored,
}

/// Resource to track the current score
#[derive(Resource)]
pub struct Score {
    p1: u32,
    p2: u32,
    serve_count: u32,   // Points since last serve switch
    server_is_p1: bool, // true if P1 is serving
}

impl Default for Score {
    fn default() -> Self {
        Self {
            p1: 0,
            p2: 0,
            serve_count: 0,
            // Randomly choose initial server
            server_is_p1: rand::thread_rng().gen_bool(0.5),
        }
    }
}

impl Score {
    /// Updates score and determines serve based on table tennis rules
    fn add_point(&mut self, p1_scored: bool) {
        // Update score
        if p1_scored {
            self.p1 += 1;
        } else {
            self.p2 += 1;
        }

        self.serve_count += 1;

        // Determine if we need to switch servers
        let in_deuce = self.p1 >= 10 && self.p2 >= 10;
        let switch_threshold = if in_deuce { 1 } else { 2 };

        if self.serve_count >= switch_threshold {
            self.server_is_p1 = !self.server_is_p1;
            self.serve_count = 0;
        }
    }
}

/// Creates a ball entity at the center of the board
fn create_ball(commands: &mut Commands, served_by_p1: bool) {
    let direction = if served_by_p1 { 1 } else { -1 };

    commands.spawn((
        Ball,
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(BALL_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        // Physics components
        RigidBody::Dynamic,
        Collider::ball(BALL_SIZE / 2.0),
        Velocity::linear(Vec2::new(BALL_SPEED * direction as f32, 0.0)),
        Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        },
        GravityScale(0.0),
        Ccd::enabled(),
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

/// Initial game setup: spawns ball and initializes score
fn setup_game(mut commands: Commands) {
    let score = Score::default();
    create_ball(&mut commands, score.server_is_p1);
    commands.insert_resource(score);
}

/// System that maintains constant ball velocity
fn maintain_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for mut velocity in query.iter_mut() {
        let current_velocity = velocity.linvel;
        let current_speed = current_velocity.length();

        if (current_speed - BALL_SPEED).abs() > SPEED_TOLERANCE {
            velocity.linvel = current_velocity.normalize() * BALL_SPEED;
        }
    }
}

/// System that handles scoring when ball hits walls
fn handle_scoring(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut score_events: EventWriter<ScoreEvent>,
    mut score: ResMut<Score>,
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
                        score_events.send(ScoreEvent::P2Scored);
                        commands.entity(ball_entity).despawn();
                        score.add_point(false);
                        create_ball(&mut commands, score.server_is_p1);
                    }
                    Wall::Right => {
                        score_events.send(ScoreEvent::P1Scored);
                        commands.entity(ball_entity).despawn();
                        score.add_point(true);
                        create_ball(&mut commands, score.server_is_p1);
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreEvent>()
            .init_resource::<Score>()
            .add_systems(Startup, setup_game)
            .add_systems(Update, (maintain_ball_velocity, handle_scoring));
    }
}
