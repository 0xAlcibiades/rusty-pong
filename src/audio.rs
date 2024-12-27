use bevy::app::{App, Plugin, Update};
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, OnEnter, OnExit, ParamSet, Res, ResMut, Resource};
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioPlugin, AudioTween};
use crate::GameState;

/// Plugin that manages background music functionality, including playing, pausing,
/// and toggling music based on game state and user input.
pub struct MusicPlugin;

/// Resource that tracks the current state of the background music
#[derive(Resource)]
struct MusicState {
    /// Whether the music is currently enabled
    playing: bool,
    /// Handle to the current audio instance, if one exists
    handle: Option<Handle<AudioInstance>>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            playing: false, // Start with music disabled
            handle: None,
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .init_resource::<MusicState>()
            // Add system to handle M key press for toggling music
            .add_systems(Update, handle_music_toggle)
            // Add systems to handle music state changes based on game state
            .add_systems(OnEnter(GameState::Playing), start_background_music)
            .add_systems(OnExit(GameState::Playing), pause_background_music)
            .add_systems(OnEnter(GameState::Paused), pause_background_music)
            .add_systems(OnExit(GameState::Paused), resume_background_music);
    }
}

/// Starts playing background music if it's not already playing
fn start_background_music(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
) {
    if !music_state.playing {
        // Load and play the audio file in a loop
        let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
        music_state.handle = Some(handle);
        music_state.playing = true;
    }
}

/// Pauses the currently playing background music
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

/// Resumes the background music if it was previously playing
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

/// Handles toggling the background music on/off when the M key is pressed
fn handle_music_toggle(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut param_set: ParamSet<(ResMut<MusicState>, ResMut<Assets<AudioInstance>>)>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        // Toggle the playing state
        let playing = {
            let mut music_state = param_set.p0();
            music_state.playing = !music_state.playing;
            music_state.playing
        };

        if playing {
            // Start playing new background music
            let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
            param_set.p0().handle = Some(handle);
        } else {
            // Stop the current background music
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