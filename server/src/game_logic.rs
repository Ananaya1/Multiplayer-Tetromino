//! Core game logic (server-only, supports multiple clients)

use crate::there_is_collision;

use crate::*;
use bevy::prelude::*;
use common::board::*;
use common::data::*;
use lightyear::prelude::server::*;

/// Handle the input received from the client
pub fn handle_user_input(
    _trigger: On<ReceivedInput>,
    _states: Query<(&SharedGameState, &ControlledBy)>,
    _tetrominoes: Query<(&mut Tetromino, &ControlledBy), With<Active>>,
    _obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
) {
    todo!()
}

/// Drop the piece whenever the gravity timer goes off
///
/// The implementation of this system is given to you as an example.
pub fn gravity(
    states: Query<(&mut SharedGameState, &ControlledBy)>,
    active: Query<(&mut Tetromino, &ControlledBy), With<Active>>,
    obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
    time: Res<Time<Fixed>>,
) {
    let mut active = build_per_client_table(active);
    let obstacles = build_per_client_lists(obstacles);
    let states = build_per_client_table(states);

    for (client, mut state) in states {
        let Some(tetromino) = active.get_mut(&client) else {
            warn!("No tetromino for client {client:?}");
            continue;
        };
        let empty_vec = vec![];
        let obstacles = obstacles.get(&client).unwrap_or(&empty_vec);

        state.gravity_timer.tick(time.delta());
        let mut new_tetromino = *(*tetromino);

        if state.gravity_timer.just_finished() {
            new_tetromino.shift(0, -1);
        }
        if new_tetromino != **tetromino
            && !there_is_collision(&new_tetromino, obstacles.iter().copied())
        {
            **tetromino = new_tetromino;
        }
    }
}

/// Check if the active tetromino cannot move down. If so, deactivate it.
pub fn deactivate_if_stuck(
    _commands: Commands,
    _peer_ids: Query<&RemoteId, With<ClientOf>>,
    // figure out and add other system parameters
) {
    todo!()
}

/// Spawn the next tetromino if there is no active tetromino.  This should also
/// update the next tetromino window.
pub fn spawn_next_tetromino(
    _commands: Commands,
    _peer_ids: Query<&RemoteId, With<ClientOf>>,
    // figure out and add other system parameters
) {
    todo!()
}

/// A system to detect the full lines (lines containing only obstacles and no
/// empty space) and to delete them.
///
/// After the lines are deleted, any obstacles above the deleted lines should be
/// moved down using naive gravity (the obstacles move down only by the number
/// of lines below them that are deleted).
pub fn delete_full_lines(
    _obstacles: Query<((Entity, &mut Block), &ControlledBy), With<Obstacle>>,
    _commands: Commands,
) {
    todo!()
}

/// Swap the current piece and the piece in the hold window on user input.
///
/// If no piece is held, then take the next piece as the active piece and move
/// the current piece to the hold window.
///
/// This system also has to make sure that the swap is legal and kick the piece
/// up by up to 4 times until the swap is legal.  If that is not possible, then
/// abort the swap.
pub fn swap_hold(
    _trigger: On<ReceivedInput>,
    _commands: Commands,
    _peer_ids: Query<&RemoteId, With<ClientOf>>,
    // figure out and add other system parameters
) {
    todo!()
}

/// Update hard drop state based on user input
pub fn update_hard_drop(
    _trigger: On<ReceivedInput>,
    _states: Query<(&mut SharedGameState, &ControlledBy)>,
) {
    todo!()
}
