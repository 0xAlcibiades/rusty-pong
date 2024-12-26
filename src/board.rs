use bevy::app::Plugin;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite::Sprite;
use bevy_rapier2d::prelude::*;

/// Component that identifies which wall this entity represents
#[derive(Component)]
pub enum Wall {
    Top,
    Bottom,
    Left,
    Right,
}

/// Physical dimensions of the game board and its elements
const WALL_THICKNESS: f32 = 0.1; // Wall thickness in world units
const BOARD_WIDTH: f32 = 16.0; // Total width of game board
const BOARD_HEIGHT: f32 = 10.0; // Total height of game board

/// Center line visual settings
const DASH_LENGTH: f32 = 0.8; // Length of each dash
const DASH_WIDTH: f32 = 0.1; // Width of each dash
const DASH_GAP: f32 = 0.4; // Gap between dashes

/// Physics settings for the walls
const WALL_RESTITUTION: f32 = 1.5; // Wall bounciness (>1 means adding energy)

/// Creates the black background color resource
pub fn black_background() -> ClearColor {
    ClearColor(Color::srgb(0.0, 0.0, 0.0))
}

/// Common physics bundle for walls to ensure consistent behavior
fn wall_physics_bundle(
    width: f32,
    height: f32,
) -> (
    RigidBody,
    Collider,
    Restitution,
    Friction,
    ActiveCollisionTypes,
    ActiveEvents,
) {
    (
        RigidBody::Fixed, // Walls don't move
        Collider::cuboid(width / 2.0, height / 2.0),
        Restitution {
            coefficient: WALL_RESTITUTION,
            combine_rule: CoefficientCombineRule::Max, // Use highest restitution in collisions
        },
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min, // No friction to maintain energy
        },
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
    )
}

/// Spawns the center line made up of dashed sprites.
/// This is purely visual and has no collision components.
fn spawn_center_line(mut commands: Commands) {
    // Calculate space for one complete dash cycle
    let dash_cycle = DASH_LENGTH + DASH_GAP;

    // Calculate number of complete cycles that fit
    let num_cycles = (BOARD_HEIGHT / dash_cycle).floor();

    // Center the pattern vertically
    let total_pattern_height = num_cycles * dash_cycle - DASH_GAP;
    let start_y = -(total_pattern_height / 2.0);

    // Spawn visual dashes
    for i in 0..num_cycles as i32 {
        let y_position = start_y + (i as f32 * dash_cycle) + (DASH_LENGTH / 2.0);

        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(DASH_WIDTH, DASH_LENGTH)),
                ..default()
            },
            Transform::from_xyz(0.0, y_position, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }
}

/// Spawns the four walls that make up the game board boundaries.
/// Each wall is given bouncy physics properties to create more
/// interesting ball trajectories.
fn spawn_walls(mut commands: Commands) {
    let half_width = BOARD_WIDTH / 2.0;
    let half_height = BOARD_HEIGHT / 2.0;

    // Top wall
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BOARD_WIDTH, WALL_THICKNESS)),
            ..default()
        },
        Transform::from_xyz(0.0, half_height, 0.0),
        wall_physics_bundle(BOARD_WIDTH, WALL_THICKNESS),
        Wall::Top,
    ));

    // Bottom wall
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(BOARD_WIDTH, WALL_THICKNESS)),
            ..default()
        },
        Transform::from_xyz(0.0, -half_height, 0.0),
        wall_physics_bundle(BOARD_WIDTH, WALL_THICKNESS),
        Wall::Bottom,
    ));

    // Left wall (scoring wall for P2)
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(WALL_THICKNESS, BOARD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-half_width, 0.0, 0.0),
        wall_physics_bundle(WALL_THICKNESS, BOARD_HEIGHT),
        Wall::Left,
    ));

    // Right wall (scoring wall for P1)
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(WALL_THICKNESS, BOARD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(half_width, 0.0, 0.0),
        wall_physics_bundle(WALL_THICKNESS, BOARD_HEIGHT),
        Wall::Right,
    ));
}

/// Plugin that manages the game board setup
/// Responsible for:
/// - Creating the black background
/// - Spawning the bouncy walls
/// - Drawing the center line
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(black_background())
            .add_systems(Startup, (spawn_walls, spawn_center_line));
    }
}
