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
const BALL_SPEED: f32 = 10.0; // Constant speed the ball should maintain
const SPEED_TOLERANCE: f32 = 0.5; // Allowed speed variation before correction
const RESTITUTION: f32 = 1.5; // Bounciness (>1 means gaining energy)
const BALL_MASS: f32 = 0.1; // Mass affects collision response

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
    // Determine initial direction based on server
    let direction = if served_by_p1 { 1 } else { -1 };

    commands.spawn((
        // Identification and visual components
        Ball,                                                             // Marker component
        Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))),                 // Visual shape
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))), // Color
        Transform::from_xyz(0.0, 0.0, 0.0),                               // Start at center
        // Physics body configuration
        RigidBody::Dynamic,              // Moves based on physics
        Collider::ball(BALL_SIZE / 2.0), // Circular collision shape
        // Initial velocity (horizontal only)
        Velocity::linear(Vec2::new(BALL_SPEED * direction as f32, 0.0)),
        // Collision response properties
        Restitution {
            coefficient: RESTITUTION,
            combine_rule: CoefficientCombineRule::Max, // Use highest restitution in collisions
        },
        Friction {
            coefficient: 0.0,                          // Frictionless
            combine_rule: CoefficientCombineRule::Min, // Use lowest friction in collisions
        },
        // Physics behavior modifiers
        Damping {
            linear_damping: 0.0,  // No speed loss over time
            angular_damping: 0.0, // No rotation slowdown
        },
        GravityScale(0.0), // Ignore gravity
        // Collision detection settings
        Ccd::enabled(),                            // Continuous collision detection
        ActiveCollisionTypes::all(),               // Detect all collision types
        ActiveEvents::COLLISION_EVENTS,            // Generate collision events
        AdditionalMassProperties::Mass(BALL_MASS), // Set specific mass
    ));
}

/// System to clean up the ball when exiting the Playing state.
/// This prevents the ball from persisting in other game states.
fn cleanup_ball(mut commands: Commands, ball_query: Query<Entity, With<Ball>>) {
    for entity in ball_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// System that maintains constant ball speed regardless of collisions.
///
/// This is necessary because:
/// 1. Collisions can change the ball's speed
/// 2. We want the game to maintain a consistent pace
/// 3. Numerical errors can accumulate over time
///
/// The system checks the current speed against BALL_SPEED and
/// corrects it if it deviates beyond SPEED_TOLERANCE.
fn maintain_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for mut velocity in query.iter_mut() {
        let current_velocity = velocity.linvel;
        let current_speed = current_velocity.length();

        // Correct speed if it's outside tolerance range
        if (current_speed - BALL_SPEED).abs() > SPEED_TOLERANCE {
            // Maintain direction but normalize speed
            velocity.linvel = current_velocity.normalize() * BALL_SPEED;
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
            // Maintain ball velocity during gameplay
            .add_systems(
                Update,
                maintain_ball_velocity.run_if(in_state(GameState::Playing)),
            );
    }
}
