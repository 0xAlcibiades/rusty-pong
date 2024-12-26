use bevy::app::Plugin;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite::Sprite;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
/// Identifies which wall this entity represents
pub enum Wall {
    Top,
    Bottom,
    Left,
    Right,
}

/// Thickness of the board walls in world units.
const WALL_THICKNESS: f32 = 0.1;

/// Total width of the game board in world units.
const BOARD_WIDTH: f32 = 16.0;

/// Total height of the game board in world units.
const BOARD_HEIGHT: f32 = 10.0;

/// Configuration for the center line's dashes
const DASH_LENGTH: f32 = 0.8; // Length of each dash
const DASH_WIDTH: f32 = 0.1; // Width of each dash
const DASH_GAP: f32 = 0.4; // Gap between dashes

/// Creates the background color resource for the game.
pub fn black_background() -> ClearColor {
    ClearColor(Color::srgb(0.0, 0.0, 0.0))
}

/// Spawns the center line made up of dashed sprites.
/// This is purely visual and has no collision components.
fn spawn_center_line(mut commands: Commands) {
    // Calculate the space taken by one dash + one gap
    let dash_cycle = DASH_LENGTH + DASH_GAP;

    // Calculate how many complete dash+gap cycles fit in the board height
    let num_cycles = (BOARD_HEIGHT / dash_cycle).floor();

    // Calculate remaining space and adjust starting position to center the pattern
    let total_pattern_height = num_cycles * dash_cycle - DASH_GAP; // Subtract final gap
    let start_y = -(total_pattern_height / 2.0);

    // Spawn dashes
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
        Collider::cuboid(half_width, WALL_THICKNESS / 2.0),
        RigidBody::Fixed,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
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
        Collider::cuboid(half_width, WALL_THICKNESS / 2.0),
        RigidBody::Fixed,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
        Wall::Bottom,
    ));

    // Left wall
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(WALL_THICKNESS, BOARD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-half_width, 0.0, 0.0),
        Collider::cuboid(WALL_THICKNESS / 2.0, half_height),
        RigidBody::Fixed,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
        Wall::Left,
    ));

    // Right wall
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(WALL_THICKNESS, BOARD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(half_width, 0.0, 0.0),
        Collider::cuboid(WALL_THICKNESS / 2.0, half_height),
        RigidBody::Fixed,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
        Wall::Right,
    ));
}

/// Plugin responsible for setting up the game board.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(black_background())
            .add_systems(Startup, (spawn_walls, spawn_center_line));
    }
}
