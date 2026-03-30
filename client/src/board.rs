//! The tetris board setup


use crate::ui::{BG_COLOR, PADDING};

use bevy::prelude::*;
use common::board::*;
use common::data::*;

// Create a logical tile to insert into a board.
//
// Board width and board height are the information of the board this tile is
// placed in.
fn mk_tile(
    x: u32,
    y: u32,
    board_width: u32,
    board_height: u32,
    tile_mesh: Handle<Mesh>,
    tile_material: Handle<ColorMaterial>,
) -> impl Bundle {
    (
        Mesh2d(tile_mesh),
        MeshMaterial2d(tile_material),
        (
            Text2d(format!("{x},{y}")),
            TextFont {
                font_size: 12.0,
                ..Default::default()
            },
        ),
        Transform::from_xyz(
            (x as f32 - board_width as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            (y as f32 - board_height as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            0.0,
        )
        .with_scale(Vec3::splat(0.8)),
    )
}

/// The calculated window height based on the board size.
pub const fn window_height() -> f32 {
    TILE_SIDE_LEN * BOARD_HEIGHT as f32 + PADDING * 2.0 + crate::ui::TEXT_SIZE * 2.0
}

/// The calculated window width based on the board size.
pub const fn window_width() -> f32 {
    window_height()
}

/// Set up a side window to show the next piece or the hold area.
pub fn spawn_side_window(
    transform: Transform,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    commands: &mut Commands,
    title: &str,
    marker: impl Component + Copy,
) {
    commands
        .spawn((transform, Visibility::default()))
        .with_children(|parent| {
            (0..5).for_each(|y| {
                (0..5).for_each(|x| {
                    parent.spawn((
                        mk_tile(x, y, 5, 5, mesh.clone(), material.clone()),
                        Block {
                            cell: Cell(x as i32, y as i32),
                            color: BG_COLOR,
                        },
                        marker,
                    ));
                })
            });

            parent.spawn((
                Transform::from_xyz(-4. * TILE_SIDE_LEN * 0.5, 5. * TILE_SIDE_LEN * 0.5, -1.0),
                Text2d::new(title),
            ));
        });
}

/// Set up the window. Only used when not testing.
pub fn setup_window(mut window: Single<&mut Window>) {
    window.resolution.set(window_height(), window_height());
}

/// Create the board and initialize game data
pub fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Background color
    commands.insert_resource(ClearColor(Color::BLACK));

    // Set up the camera
    commands.spawn(Camera2d);

    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);

    // Set up the board
    let _cells = (0..BOARD_HEIGHT)
        .map(|y| {
            (0..BOARD_WIDTH)
                .map(|x| {
                    commands
                        .spawn((
                            mk_tile(
                                x,
                                y,
                                BOARD_WIDTH,
                                BOARD_HEIGHT,
                                mesh.clone(),
                                material.clone(),
                            ),
                            Block {
                                cell: Cell(x as i32, y as i32),
                                color: BG_COLOR,
                            },
                            MainBoard,
                        ))
                        .id()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            window_height() * 0.5 - 5.0 * TILE_SIDE_LEN * 0.5 - PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Next",
        Next,
    );
}

/// Create the hold window
pub fn setup_hold_window(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);
    // held window
    crate::board::spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            -window_height() * 0.5 + 5.0 * TILE_SIDE_LEN * 0.5 + PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Hold",
        Hold,
    );
}

/// Read the keyboard input and send it to the server as a message.
pub fn handle_user_input() {
    todo!()
}

/// Redraw the board.
pub fn redraw_board(
    _commands: Commands,
    _materials: ResMut<Assets<ColorMaterial>>,
    _tetrominoes: Query<&Tetromino, With<Active>>,
    _obstacles: Query<&Block, With<Obstacle>>,
    // note: these are distinct from the obstacles
    _board: Query<(&Block, Entity), With<MainBoard>>,
) {
    todo!()
}

/// Redraw the side board with the given marker component.
pub fn redraw_side_board<Marker: Component>() {
    todo!()
}
