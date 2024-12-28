//! Ball Physics Module
//!
//! This module handles all aspects of the game ball, including:
//! - Physical properties and behavior
//! - Spawn and cleanup logic
//! - Velocity maintenance
//! - Collision detection and response
//!
//! The ball uses Rapier2D physics for realistic movement and collisions.

use crate::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Physical properties of the ball that define its behavior
const BALL_SIZE: f32 = 0.3; // Diameter in world units
const MIN_VELOCITY: f32 = 7.0; // Minimum speed for the ball
const MAX_VELOCITY: f32 = 32.0; // Maximum speed for the ball
const SPEED_TOLERANCE: f32 = 3.0; // Allowed speed variation before correction
const RESTITUTION: f32 = 0.9; // Bounciness (>1 means gaining energy)
const BALL_MASS: f32 = 0.0027; // Mass affects collision response

/// Marker component to identify ball entities.
/// Used for querying and managing ball-specific behavior.
#[derive(Component)]
pub struct Ball;

/// Creates a new ball entity with all necessary components for physics and rendering.
///
/// # Arguments
/// * `commands` - Command buffer for entity creation
/// * `meshes` - Asset storage for the ball's mesh
/// * `materials` - Asset storage for the ball's material
/// * `served_by_p1` - Direction ball should initially move (true = right, false = left)
pub fn create_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    served_by_p1: bool,
) {
    let direction = if served_by_p1 { 1 } else { -1 };
    let initial_velocity = Vec2::new(MIN_VELOCITY * direction as f32, 0.0);

    commands
        .spawn(Ball)
        // Visual components
        .insert(Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))))
        .insert(MeshMaterial2d(
            materials.add(ColorMaterial::from(Color::WHITE)),
        ))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        // Physics body setup
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(BALL_SIZE / 2.0))
        .insert(Velocity::linear(initial_velocity))
        // Collision properties
        .insert(Restitution {
            coefficient: RESTITUTION,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        // Physics modifiers
        .insert(Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        })
        .insert(GravityScale(0.0))
        // Collision detection
        .insert(Ccd::enabled())
        .insert(Sleeping::disabled())
        .insert(ActiveCollisionTypes::all())
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(AdditionalMassProperties::Mass(BALL_MASS));
}

/// System to clean up the ball when exiting the Playing state.
/// This prevents the ball from persisting in other game states.
fn cleanup_ball(mut commands: Commands, ball_query: Query<Entity, With<Ball>>) {
    for entity in ball_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn maintain_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for mut velocity in query.iter_mut() {
        let current_velocity = velocity.linvel;
        let current_speed = current_velocity.length();

        // Check absolute magnitude and adjust if needed
        if current_speed != 0.0 {  // Prevent division by zero
            let new_speed = if current_speed.abs() < MIN_VELOCITY {
                MIN_VELOCITY
            } else if current_speed.abs() > MAX_VELOCITY {
                MAX_VELOCITY
            } else {
                current_speed
            };

            velocity.linvel = current_velocity.normalize() * new_speed;
        }
    }
}

/// Plugin that manages all ball-related systems.
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
            // Clean up ball when leaving Playing state
            .add_systems(OnExit(GameState::Playing), cleanup_ball)
            .add_systems(Update, maintain_ball_velocity);
    }
}
