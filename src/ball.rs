use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Physical properties of the ball
const BALL_SIZE: f32 = 0.3; // Diameter in world units
const BALL_SPEED: f32 = 10.0; // Velocity in world units per second
const SPEED_TOLERANCE: f32 = 0.5; // Allowed deviation from BALL_SPEED
const RESTITUTION: f32 = 1.5; // Bounciness (>1 means gaining energy on bounce)

const BALL_MASS: f32 = 0.1; // Make the ball very light!

/// Component marking an entity as the game ball.
/// Required for querying ball entities in physics and game systems.
#[derive(Component)]
pub struct Ball;

/// Creates a new ball entity at the center of the board
///
/// # Arguments
/// * `commands` - Command buffer for entity creation
/// * `served_by_p1` - If true, ball moves right (served by P1), if false, moves left (served by P2)
///
/// The ball is configured with:
/// - High restitution for energetic bounces
/// - Zero friction to maintain momentum
/// - Zero damping to prevent velocity loss
/// - Continuous collision detection to prevent tunneling
pub fn create_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    served_by_p1: bool,
) {
    let direction = if served_by_p1 { 1 } else { -1 };

    commands.spawn((
        Ball,
        // Visual components using new non-deprecated approach
        Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        // Physics components remain the same
        RigidBody::Dynamic,
        Collider::ball(BALL_SIZE / 2.0),
        Velocity::linear(Vec2::new(BALL_SPEED * direction as f32, 0.0)),
        Restitution {
            coefficient: RESTITUTION,
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
        AdditionalMassProperties::Mass(BALL_MASS),
    ));
}

/// Spawns the initial ball moving towards Player 1
fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    create_ball(&mut commands, &mut meshes, &mut materials, false);
}

/// System that maintains constant ball velocity
///
/// This system ensures the ball maintains a consistent speed even after
/// multiple collisions. Physics engines can sometimes introduce small
/// velocity changes due to numerical precision issues, so this system
/// corrects any significant deviations from the desired speed.
///
/// The correction only happens if the speed differs from BALL_SPEED
/// by more than SPEED_TOLERANCE to avoid constant minor adjustments.
fn maintain_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for mut velocity in query.iter_mut() {
        let current_velocity = velocity.linvel;
        let current_speed = current_velocity.length();

        // Only adjust if speed has deviated significantly
        if (current_speed - BALL_SPEED).abs() > SPEED_TOLERANCE {
            // Normalize to get direction, then scale to desired speed
            velocity.linvel = current_velocity.normalize() * BALL_SPEED;
        }
    }
}

/// Plugin that handles all ball-related functionality
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_game)
            .add_systems(Update, maintain_ball_velocity);
    }
}
