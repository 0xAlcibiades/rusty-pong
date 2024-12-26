use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{Camera2d, Commands, OrthographicProjection};
use bevy::render::camera::ScalingMode;

/// Spawns a 2D camera with a fixed vertical viewport height.
///
/// This camera is set up for orthographic projection, which means:
/// - No perspective distortion (parallel lines stay parallel)
/// - Objects maintain their size regardless of distance from camera
/// - Coordinates directly map to world units
///
/// The camera uses a fixed vertical height of 10 units, meaning:
/// - The viewport will always be 10 units tall in world space
/// - The width will adjust based on the window aspect ratio
/// - (0,0) is at the center of the screen
/// - Y ranges from -5 to +5 vertically
/// - X range depends on the aspect ratio (e.g., -8 to +8 for a 16:10 window)
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        // Camera2d bundle marks this as a 2D camera
        Camera2d,
        // Configure the orthographic projection
        OrthographicProjection {
            // Use fixed vertical scaling mode
            // This means the viewport will always be exactly 10 units tall,
            // and the width will adjust based on the window's aspect ratio
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10., // Height in world units
            },
            // Use the default settings for everything else
            ..OrthographicProjection::default_2d()
        },
    ));
}

/// Plugin that handles camera setup for the game.
///
/// This plugin is responsible for spawning and configuring the main 2D camera
/// that will be used to render the game world. It sets up an orthographic
/// projection with a fixed vertical height, which is ideal for 2D games
/// where you want consistent sizing regardless of screen dimensions.
pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Add the camera spawn system to the startup schedule,
        // ensuring it runs when the game starts
        app.add_systems(Startup, spawn_camera);
    }
}
