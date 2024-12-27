use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// Plugin that handles background music functionality.
pub struct MusicPlugin;

#[derive(Resource)]
struct MusicState {
    playing: bool,
    handle: Option<Handle<AudioInstance>>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            playing: false, // Start with music paused.
            handle: None,
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .init_resource::<MusicState>()
            .add_systems(Update, handle_music_toggle)
            .add_systems(OnEnter(GameState::Playing), start_background_music)
            .add_systems(OnExit(GameState::Playing), pause_background_music)
            .add_systems(OnEnter(GameState::Paused), pause_background_music)
            .add_systems(OnExit(GameState::Paused), resume_background_music);
    }
}

fn start_background_music(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
) {
    if !music_state.playing {
        let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
        music_state.handle = Some(handle);
        music_state.playing = true;
    }
}

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

fn handle_music_toggle(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut param_set: ParamSet<(ResMut<MusicState>, ResMut<Assets<AudioInstance>>)>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        let playing = {
            let mut music_state = param_set.p0();
            music_state.playing = !music_state.playing;
            music_state.playing
        };

        if playing {
            let handle = audio.play(asset_server.load("pong.flac")).looped().handle();
            param_set.p0().handle = Some(handle);
        } else {
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
