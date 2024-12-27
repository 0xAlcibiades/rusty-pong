use bevy::prelude::WindowPlugin;
use bevy::window::Window;

/// Creates and returns a window plugin configured for browser-based deployment.
///
/// This function provides a `WindowPlugin` with settings optimized for web deployment,
/// ensuring the game canvas behaves appropriately within the browser environment.
///
/// # Window Configuration
/// - The game canvas will automatically resize to fill its parent element
/// - Browser keyboard shortcuts remain functional (F5 for refresh, etc.)
/// - Other window settings use their default values
///
/// # Example Parent HTML
/// ```html
/// <div style="width: 100%; height: 100%">
///     <!-- The game canvas will fill this div -->
/// </div>
/// ```
///
/// # Returns
/// A `WindowPlugin` instance with browser-specific configurations.
pub(crate) fn default_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            // Set the canvas ID to match the parent element
            canvas: Some("#pong-game-canvas".into()),
            // Enable canvas-to-parent fitting
            // This makes the game canvas automatically resize to fill its parent element,
            // providing a responsive layout that works across different screen sizes
            fit_canvas_to_parent: true,

            // Allow default browser event handling
            // When false, this would prevent browser shortcuts from working.
            // We set it to false to maintain expected browser behavior:
            // - F5: Page refresh
            // - F12: Developer tools
            // - Ctrl+R: Refresh
            // - Other standard browser shortcuts
            prevent_default_event_handling: false,

            // Use defaults for all other window settings
            // This includes:
            // - Title
            // - Resolution
            // - Position
            // - Decorations
            // - etc.
            ..Default::default()
        }),
        // Use defaults for any other plugin settings
        ..Default::default()
    }
}
