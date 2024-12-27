//! Audio Management Module
//!
//! This module handles the game's audio system, specifically the background music.
//! It provides functionality to play, pause, and toggle background music using the 'M' key.
//! The module uses Bevy's audio system to manage sound playback.

use bevy::{
    asset::AssetServer,
    audio::{AudioPlayer, PlaybackSettings},
    prelude::*,
};

/// Plugin that handles background music functionality.
///
/// This plugin initializes the music system and sets up the necessary systems
/// for music playback control. It manages:
/// - Initial music playback on game start
/// - Music toggle functionality (M key)
/// - Music state tracking
pub struct MusicPlugin;

/// Resource that tracks the state of the background music.
///
/// This resource maintains the current state of the music system, including
/// whether music is playing and a reference to the current music player entity.
#[derive(Resource)]
struct MusicState {
    /// Whether the music is currently playing
    playing: bool,
    /// The entity ID of the current music player.
    /// This is None when music is stopped or not yet started.
    entity: Option<Entity>,
}

impl Default for MusicState {
    /// Creates the default state for music:
    /// - Music enabled by default (playing = true)
    /// - No active music player (entity = None)
    fn default() -> Self {
        Self {
            playing: true,
            entity: None,
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize the music state resource
            .init_resource::<MusicState>()
            // Start playing music when the game launches
            .add_systems(Startup, start_background_music)
            // Add system to handle music toggling during gameplay
            .add_systems(Update, handle_music_toggle);
    }
}

/// Starts playing the background music when the game launches.
///
/// This system runs during startup and:
/// 1. Creates a new music player entity
/// 2. Starts playing the background music
/// 3. Stores the music player entity in the MusicState
fn start_background_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
) {
    let entity = spawn_music_player(&mut commands, &asset_server);
    music_state.entity = Some(entity);
}

/// System that handles toggling music on/off with the 'M' key.
///
/// This system:
/// 1. Detects when the M key is pressed
/// 2. Toggles the music state between playing and stopped
/// 3. Either spawns a new music player or despawns existing ones
/// 4. Updates the MusicState resource accordingly
fn handle_music_toggle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut music_state: ResMut<MusicState>,
    mut audio_query: Query<Entity, With<AudioPlayer>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        // Toggle the playing state
        music_state.playing = !music_state.playing;

        if music_state.playing {
            // If music should be playing, spawn a new player
            let entity = spawn_music_player(&mut commands, &asset_server);
            music_state.entity = Some(entity);
        } else {
            // If music should be stopped, despawn all players
            despawn_music_players(&mut commands, &mut audio_query);
            music_state.entity = None;
        }
    }
}

/// Helper function to spawn a new music player entity.
///
/// Creates a new entity with:
/// - An AudioPlayer component configured to play "pong.mp3"
/// - PlaybackSettings set to loop the audio
///
/// Returns the Entity ID of the newly created music player.
fn spawn_music_player(commands: &mut Commands, asset_server: &AssetServer) -> Entity {
    commands
        .spawn((
            // Create new audio player with the background music
            AudioPlayer::new(asset_server.load("pong.mp3")),
            // Configure it to loop continuously
            PlaybackSettings::LOOP,
        ))
        .id()
}

/// Helper function to despawn all music player entities.
///
/// This function:
/// 1. Queries for all entities with an AudioPlayer component
/// 2. Despawns each one, effectively stopping all music playback
fn despawn_music_players(
    commands: &mut Commands,
    audio_query: &mut Query<Entity, With<AudioPlayer>>,
) {
    for entity in audio_query.iter_mut() {
        commands.entity(entity).despawn();
    }
}
