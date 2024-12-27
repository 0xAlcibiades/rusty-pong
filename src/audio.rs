use crate::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, OnEnter, OnExit, ParamSet, Res, ResMut, Resource};
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioPlugin, AudioTween};

/// The MusicPlugin manages all background music functionality for the game.
///
/// This plugin handles:
/// - Playing background music during gameplay
/// - Pausing/resuming music based on game state
/// - Toggling music on/off with the 'M' key
/// - Managing the music state across game state transitions
pub struct MusicPlugin;

/// Tracks the current state of the background music system.
///
/// This resource maintains information about:
/// - Whether music is currently enabled
/// - The handle to the current audio instance (if one exists)
///
/// The state persists across game state changes to maintain user preferences
/// for music playback.
#[derive(Resource)]
struct MusicState {
    /// Indicates if music should be playing (true) or muted (false)
    playing: bool,
    /// Optional handle to the current audio instance
    /// None if no music has been started or if music was explicitly stopped
    handle: Option<Handle<AudioInstance>>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            playing: false, // Start with music disabled by default
            handle: None,   // No audio instance at initialization
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .init_resource::<MusicState>()
            // System to handle manual music toggling via 'M' key
            .add_systems(Update, handle_music_toggle)
            // Systems to manage music across different game states
            .add_systems(OnEnter(GameState::Playing), start_background_music)
            .add_systems(OnExit(GameState::Playing), pause_background_music)
            .add_systems(OnEnter(GameState::Paused), pause_background_music)
            .add_systems(OnExit(GameState::Paused), resume_background_music);
    }
}

/// Initiates background music playback if it's not already playing.
///
/// This system:
/// 1. Checks if music is currently disabled
/// 2. Loads the music asset
/// 3. Starts playback in a looped configuration
/// 4. Stores the audio handle for future reference
fn start_background_music(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
) {
    if !music_state.playing {
        // Create a new looped audio instance and store its handle
        let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
        music_state.handle = Some(handle);
        music_state.playing = true;
    }
}

/// Temporarily pauses the background music without changing the enabled state.
///
/// Used when:
/// - The game is paused
/// - Transitioning between game states where music should be temporarily stopped
fn pause_background_music(
    music_state: ResMut<MusicState>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(handle) = &music_state.handle {
        if let Some(instance) = audio_instances.get_mut(handle) {
            instance.pause(AudioTween::default());
        }
    }
}

/// Resumes background music playback if it was previously enabled.
///
/// This system:
/// 1. Checks if music should be playing based on the stored state
/// 2. If enabled, resumes playback of the existing audio instance
fn resume_background_music(
    music_state: ResMut<MusicState>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if music_state.playing {
        if let Some(handle) = &music_state.handle {
            if let Some(instance) = audio_instances.get_mut(handle) {
                instance.resume(AudioTween::default());
            }
        }
    }
}

/// Manages toggling the background music on/off via the 'M' key.
///
/// This system:
/// 1. Detects 'M' key presses
/// 2. Toggles the music state
/// 3. Either starts new music playback or stops the current playback
/// 4. Updates the MusicState resource accordingly
///
/// Uses ParamSet to safely handle multiple mutable resources:
/// - p0: MusicState for tracking playback state
/// - p1: AudioInstances for controlling actual playback
fn handle_music_toggle(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut param_set: ParamSet<(ResMut<MusicState>, ResMut<Assets<AudioInstance>>)>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        // Toggle the playing state in a separate scope to release the borrow
        let playing = {
            let mut music_state = param_set.p0();
            music_state.playing = !music_state.playing;
            music_state.playing
        };

        if playing {
            // Start new background music
            let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
            param_set.p0().handle = Some(handle);
        } else {
            // Stop current background music
            let handle = param_set.p0().handle.clone();
            if let Some(handle) = handle {
                if let Some(instance) = param_set.p1().get_mut(&handle) {
                    instance.stop(AudioTween::default());
                }
                param_set.p0().handle = None;
            }
        }
    }
}
