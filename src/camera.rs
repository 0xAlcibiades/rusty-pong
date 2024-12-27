//! Camera Module
//!
//! This module handles the game's camera system, providing a 2D orthographic view
//! of the game world. It maintains consistent scaling regardless of window size
//! by using a fixed vertical height strategy.
//!
//! The camera system ensures that:
//! - Game objects appear the same size regardless of screen dimensions
//! - The game viewport adjusts properly to different aspect ratios
//! - World coordinates map consistently to screen space

use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2d, Commands, OrthographicProjection};
use bevy::render::camera::ScalingMode;

/// Spawns a 2D camera with a fixed vertical viewport height.
///
/// # Camera Properties
/// - Uses orthographic projection for 2D rendering
/// - Maintains fixed vertical height of 10 world units
/// - Automatically adjusts width based on window aspect ratio
/// - Centers coordinate system at (0,0)
///
/// # Coordinate System
/// The viewport coordinates are mapped as follows:
/// - Center: (0, 0)
/// - Vertical range: -5 to +5 units
/// - Horizontal range: varies with aspect ratio
///   - 16:9 aspect: approximately -8.89 to +8.89 units
///   - 16:10 aspect: approximately -8 to +8 units
///   - 4:3 aspect: approximately -6.67 to +6.67 units
///
/// # Example
/// ```
/// // Object at (0,0) appears at screen center
/// // Object at (0,5) appears at top of screen
/// // Object at (4,0) appears halfway to right edge in 16:10 window
/// ```
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        // Camera2d component marks this as a 2D camera
        // This sets up appropriate defaults for 2D rendering
        Camera2d,
        // Configure the orthographic projection settings
        OrthographicProjection {
            // Use fixed vertical scaling mode to maintain consistent height
            // This ensures the game view is always exactly 10 units tall,
            // with width adjusting to maintain the window's aspect ratio
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0, // Fixed height in world units
            },

            // Use default settings for remaining properties:
            // - Near/far clipping planes
            // - Z-axis layering
            // - Viewport origin and scale
            ..OrthographicProjection::default_2d()
        },
    ));
}

/// Plugin responsible for camera setup and management.
///
/// # Features
/// - Spawns and configures the main 2D camera
/// - Sets up orthographic projection
/// - Ensures consistent scaling across different screen sizes
pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Add camera spawn system to startup schedule
        // This ensures the camera is created when the game begins
        // and before any other systems that might need it
        app.add_systems(Startup, spawn_camera);
    }
}
