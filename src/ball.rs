//! Ball Physics Module
//!
//! This module implements the core ball mechanics for the Pong game using the Bevy game engine
//! and Rapier2D physics engine. It handles all aspects of the ball's behavior including:
//!
//! - Ball creation and initialization
//! - Physics properties and collision response
//! - Velocity management and speed constraints
//! - Cleanup and state management
//! - Collision detection and event handling
//!
//! The ball uses Rapier2D's rigid body physics system for realistic movement and collisions,
//! with carefully tuned parameters to ensure engaging gameplay while maintaining physical plausibility.

use crate::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Physical properties and gameplay constants for the ball
///
/// These constants define both the visual and physical characteristics of the ball,
/// carefully tuned to provide satisfying gameplay mechanics while maintaining
/// physical plausibility.
const BALL_SIZE: f32 = 0.3; // Ball diameter in world units (small enough for precise gameplay)
const MIN_VELOCITY: f32 = 7.0; // Minimum ball speed (ensures game keeps moving)
const MAX_VELOCITY: f32 = 20.0; // Maximum ball speed (prevents ball from becoming too fast)
const RESTITUTION: f32 = 0.9; // Bounce elasticity (slightly inelastic for better control)
const BALL_MASS: f32 = 0.0027; // Ball mass (tuned for realistic collision responses)

/// Marker component for identifying ball entities in the game world.
///
/// This component is used as a tag to:
/// - Query for ball entities in systems
/// - Filter collision events involving the ball
/// - Manage ball-specific behavior and cleanup
///
/// # Example Usage
/// ```rust
/// // Query for ball entities
/// fn ball_system(query: Query<&Transform, With<Ball>>) {
///     for transform in query.iter() {
///         // Process ball position
///     }
/// }
/// ```
#[derive(Component)]
pub struct Ball;

/// Creates a new ball entity with complete physics and rendering setup.
///
/// This function creates a ball entity configured with:
/// - Visual representation (white circle mesh)
/// - Physics body and collider
/// - Initial velocity based on serving direction
/// - Collision properties and response settings
/// - Physics modifiers for gameplay behavior
///
/// # Arguments
/// * `commands` - Command buffer for entity creation and component insertion
/// * `meshes` - Asset storage for managing the ball's visual mesh
/// * `materials` - Asset storage for managing the ball's material/color
/// * `served_by_p1` - Boolean flag indicating serve direction (true = right, false = left)
///
/// # Physics Configuration
/// The ball is configured with:
/// - Dynamic rigid body for physics-based movement
/// - Zero friction to maintain momentum
/// - Zero gravity for 2D pong mechanics
/// - Continuous collision detection for reliability
/// - Custom mass and restitution for desired bounce behavior
///
/// # Example
/// ```rust
/// create_ball(&mut commands, &mut meshes, &mut materials, true); // Serve to the right
/// ```
pub fn create_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    served_by_p1: bool,
) {
    // Calculate initial direction and velocity
    let direction = if served_by_p1 { 1 } else { -1 };
    let initial_velocity = Vec2::new(MIN_VELOCITY * direction as f32, 0.0);

    commands
        .spawn(Ball)
        // Visual Components
        // Creates a circular mesh for rendering with appropriate size
        .insert(Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))))
        // Applies white color material to the ball
        .insert(MeshMaterial2d(
            materials.add(ColorMaterial::from(Color::WHITE)),
        ))
        // Positions ball at center of screen initially
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        // Physics Body Configuration
        // Sets up dynamic rigid body for physics simulation
        .insert(RigidBody::Dynamic)
        // Creates circular collider matching visual size
        .insert(Collider::ball(BALL_SIZE / 2.0))
        // Sets initial movement velocity
        .insert(Velocity::linear(initial_velocity))
        // Collision Properties
        // Configures bounce behavior
        .insert(Restitution {
            coefficient: RESTITUTION,
            combine_rule: CoefficientCombineRule::Max,
        })
        // Removes friction for consistent movement
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        // Physics Modifiers
        // Disables velocity damping to maintain speed
        .insert(Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        })
        // Removes gravity effect
        .insert(GravityScale(0.0))
        // Collision Detection Setup
        // Enables continuous collision detection for fast movement
        .insert(Ccd::enabled())
        // Prevents physics sleep for consistent behavior
        .insert(Sleeping::disabled())
        // Enables all collision types for comprehensive detection
        .insert(ActiveCollisionTypes::all())
        // Enables collision event generation
        .insert(ActiveEvents::COLLISION_EVENTS)
        // Sets mass for collision response calculations
        .insert(AdditionalMassProperties::Mass(BALL_MASS));
}

/// System that removes the ball entity when exiting the Playing state.
///
/// This cleanup system ensures that:
/// - Ball is properly despawned when leaving gameplay
/// - No ball entities persist in other game states
/// - Memory is properly freed
/// - Game state transitions are clean
///
/// # System Parameters
/// * `commands` - Command buffer for entity manipulation
/// * `ball_query` - Query to find ball entities for cleanup
fn cleanup_ball(mut commands: Commands, ball_query: Query<Entity, With<Ball>>) {
    for entity in ball_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// System that maintains the ball's velocity within gameplay constraints.
///
/// This system ensures that:
/// - Ball never moves too slowly (maintains minimum speed)
/// - Ball never moves too quickly (caps maximum speed)
/// - Direction is preserved when adjusting speed
/// - Ball maintains consistent gameplay feel
///
/// The system runs every frame during gameplay to:
/// 1. Check current ball speed
/// 2. Compare against min/max bounds
/// 3. Adjust if necessary while preserving direction
/// 4. Handle edge cases (like zero velocity)
///
/// # Physics Notes
/// - Uses vector normalization to preserve direction
/// - Handles potential division by zero
/// - Maintains speed constraints for consistent gameplay
fn maintain_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for mut velocity in query.iter_mut() {
        let current_velocity = velocity.linvel;
        let current_speed = current_velocity.length();

        // Only adjust non-zero velocities to prevent normalization issues
        if current_speed != 0.0 {
            // Determine new speed based on constraints
            let new_speed = if current_speed.abs() < MIN_VELOCITY {
                MIN_VELOCITY // Enforce minimum speed
            } else if current_speed.abs() > MAX_VELOCITY {
                MAX_VELOCITY // Cap maximum speed
            } else {
                current_speed // Maintain current speed if within bounds
            };

            // Apply new speed while preserving direction
            velocity.linvel = current_velocity.normalize() * new_speed;
        }
    }
}

/// Plugin that manages all ball-related systems and behavior.
///
/// This plugin integrates the ball systems into the game by:
/// - Adding cleanup system for state transitions
/// - Adding velocity maintenance system for gameplay
/// - Organizing ball-related functionality
///
/// The plugin ensures proper initialization and cleanup of ball
/// mechanics while maintaining clean integration with the game's
/// state system.
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add cleanup system for state transitions
            .add_systems(OnExit(GameState::Playing), cleanup_ball)
            // Add velocity maintenance system during gameplay updates
            .add_systems(Update, maintain_ball_velocity);
    }
}
