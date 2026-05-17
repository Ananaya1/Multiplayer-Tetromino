//! Core game logic (server-only, supports multiple clients)

use crate::there_is_collision;

use crate::score::LinesCleared;
use crate::*;
use bevy::prelude::*;
use common::board::*;
use common::data::*;
use common::protocol::Input;
use lightyear::prelude::server::*;

/// Horizontal shift applied to a canonical tetromino at the active spawn.
pub const ACTIVE_SPAWN_X: i32 = (BOARD_WIDTH as i32 / 2) - 1;
/// Shift applied to a canonical tetromino in the next/hold preview window.
pub const PREVIEW_SPAWN_X: i32 = 2;
/// Shift applied to a canonical tetromino in the next/hold preview window.
pub const PREVIEW_SPAWN_Y: i32 = 2;

/// Position a canonical tetromino at the active spawn: horizontally centered,
/// with its top row on the topmost visible row.  The vertical shift depends on
/// the piece's height (the flat I piece spawns one row higher than the rest).
pub fn place_active(t: &mut Tetromino) {
    let dy = (BOARD_HEIGHT as i32 - 1) - t.bounds().top;
    t.shift(ACTIVE_SPAWN_X, dy);
}

/// Position a canonical tetromino in the preview (next/hold) window.
pub fn place_preview(t: &mut Tetromino) {
    t.shift(PREVIEW_SPAWN_X, PREVIEW_SPAWN_Y);
}

/// Handle the input received from the client
pub fn handle_user_input(
    trigger: On<ReceivedInput>,
    states: Query<(&SharedGameState, &ControlledBy)>,
    mut tetrominoes: Query<(&mut Tetromino, &ControlledBy), With<Active>>,
    obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
) {
    let client = trigger.target;
    let Some(state) = take_controlled_by(states, client) else {
        return;
    };
    let Some(mut tetromino) = take_controlled_by(tetrominoes.reborrow(), client) else {
        return;
    };
    let obstacles = filter_controlled_by(obstacles, client).collect::<Vec<_>>();
    let mut t = *tetromino;

    if trigger.inputs.0.contains(&Input::Left) {
        let mut candidate = t;
        candidate.shift(-1, 0);
        if !there_is_collision(&candidate, obstacles.iter().copied()) {
            t = candidate;
        }
    }
    if trigger.inputs.0.contains(&Input::Right) {
        let mut candidate = t;
        candidate.shift(1, 0);
        if !there_is_collision(&candidate, obstacles.iter().copied()) {
            t = candidate;
        }
    }
    if trigger.inputs.0.contains(&Input::Rotate) {
        let mut candidate = t;
        candidate.rotate();
        if !there_is_collision(&candidate, obstacles.iter().copied()) {
            t = candidate;
        }
    }
    if trigger.inputs.0.contains(&Input::Down) {
        let mut candidate = t;
        for _ in 0..state.manual_drop_gravity {
            candidate.shift(0, -1);
            if there_is_collision(&candidate, obstacles.iter().copied()) {
                break;
            }
            t = candidate;
        }
    }
    *tetromino = t;
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
    mut commands: Commands,
    peer_ids: Query<&RemoteId, With<ClientOf>>,
    time: Res<Time<Fixed>>,
    states: Query<(&mut LockdownTimer, &ControlledBy)>,
    active: Query<((Entity, &Tetromino), &ControlledBy), With<Active>>,
    obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
) {
    let active_table = build_per_client_table(active);
    let obstacle_lists = build_per_client_lists(obstacles);
    for (client, mut timer) in build_per_client_table(states) {
        let Some((active_entity, tetromino)) = active_table.get(&client).copied() else {
            timer.reset();
            continue;
        };
        let mut down = *tetromino;
        down.shift(0, -1);
        let empty = vec![];
        let client_obstacles = obstacle_lists.get(&client).unwrap_or(&empty);
        if there_is_collision(&down, client_obstacles.iter().copied()) {
            timer.start_or_advance(&time);
            if timer.just_finished() {
                commands.entity(active_entity).remove::<Active>();
                for cell in tetromino.cells {
                    commands.spawn((
                        Block {
                            cell,
                            color: tetromino.color,
                        },
                        Obstacle,
                        controlled_by(client),
                        replicate_to(client, &peer_ids),
                    ));
                }
                timer.reset();
            }
        } else {
            timer.reset();
        }
    }
}

/// Spawn the next tetromino if there is no active tetromino.  This should also
/// update the next tetromino window.
pub fn spawn_next_tetromino(
    mut commands: Commands,
    peer_ids: Query<&RemoteId, With<ClientOf>>,
    mut states: Query<(&mut PrivateGameState, &ControlledBy)>,
    active: Query<&ControlledBy, With<Active>>,
    mut next: Query<((Entity, &Tetromino), &ControlledBy), With<Next>>,
    obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
) {
    let active_clients = active
        .iter()
        .map(|c| c.owner)
        .collect::<std::collections::HashSet<_>>();
    let obstacle_lists = build_per_client_lists(obstacles);
    let mut next_map = build_per_client_table(next.reborrow());

    for (mut private, controlled_by) in &mut states {
        let client = controlled_by.owner;
        if active_clients.contains(&client) {
            continue;
        }
        let Some((next_entity, next_piece)) = next_map.remove(&client) else {
            continue;
        };
        // Recover the canonical piece from its preview position, then place it
        // at the active spawn (whose vertical offset is piece-dependent).
        let mut promoted = *next_piece;
        promoted.shift(-PREVIEW_SPAWN_X, -PREVIEW_SPAWN_Y);
        place_active(&mut promoted);
        commands.entity(next_entity).insert(promoted);
        commands.entity(next_entity).remove::<Next>().insert(Active);

        private.bag.next_tetromino();
        let mut new_next = private.bag.peek();
        place_preview(&mut new_next);
        commands.spawn((
            new_next,
            Next,
            crate::controlled_by(client),
            replicate_to(client, &peer_ids),
        ));

        let empty = Vec::new();
        if there_is_collision(
            &promoted,
            obstacle_lists
                .get(&client)
                .unwrap_or(&empty)
                .iter()
                .copied(),
        ) {
            commands.trigger(GameOver { entity: client });
        }
    }
}

/// A system to detect the full lines (lines containing only obstacles and no
/// empty space) and to delete them.
///
/// After the lines are deleted, any obstacles above the deleted lines should be
/// moved down using naive gravity (the obstacles move down only by the number
/// of lines below them that are deleted).
pub fn delete_full_lines(
    obstacles: Query<((Entity, &mut Block), &ControlledBy), With<Obstacle>>,
    mut commands: Commands,
) {
    use std::collections::{BTreeSet, HashMap};
    let by_client = build_per_client_lists(obstacles);

    for (client, mut blocks) in by_client {
        let mut row_counts: HashMap<i32, usize> = HashMap::new();
        for (_, block) in &blocks {
            *row_counts.entry(block.cell.1).or_default() += 1;
        }
        let full_rows: BTreeSet<i32> = row_counts
            .into_iter()
            .filter_map(|(row, count)| (count == BOARD_WIDTH as usize).then_some(row))
            .collect();
        if full_rows.is_empty() {
            continue;
        }
        let cleared = full_rows.len() as u32;
        for (entity, block) in &mut blocks {
            if full_rows.contains(&block.cell.1) {
                commands.entity(*entity).despawn();
            } else {
                let shift = full_rows.iter().filter(|&&row| row < block.cell.1).count() as i32;
                block.cell.1 -= shift;
            }
        }
        commands.trigger(LinesCleared {
            lines_cleared: cleared,
            target: client,
        });
    }
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
    trigger: On<ReceivedInput>,
    mut commands: Commands,
    peer_ids: Query<&RemoteId, With<ClientOf>>,
    mut states: Query<(&mut PrivateGameState, &ControlledBy)>,
    mut active: Query<((Entity, &mut Tetromino), &ControlledBy), With<Active>>,
    mut hold: Query<((Entity, &mut Tetromino), &ControlledBy), (With<Hold>, Without<Active>)>,
    mut next: Query<
        ((Entity, &mut Tetromino), &ControlledBy),
        (With<Next>, Without<Active>, Without<Hold>),
    >,
    obstacles: Query<(&Block, &ControlledBy), With<Obstacle>>,
) {
    if !trigger.inputs.0.contains(&Input::Hold) {
        return;
    }
    let client = trigger.target;

    let Some((active_entity, mut active_t)) = take_controlled_by(active.reborrow(), client) else {
        return;
    };
    let old_active = *active_t;
    let old_center = old_active.center;
    let obstacle_list: Vec<Block> = filter_controlled_by(obstacles, client).copied().collect();

    // The outgoing active piece is parked centered in the hold window.
    let mut new_hold = old_active;
    new_hold.shift(
        PREVIEW_SPAWN_X - new_hold.center.0.floor() as i32,
        PREVIEW_SPAWN_Y - new_hold.center.1.floor() as i32,
    );

    // The incoming piece is centered on the old active piece's center, then
    // kicked up by up to 4 rows until it no longer collides.
    let try_place = |mut candidate: Tetromino| -> Option<Tetromino> {
        candidate.shift(
            old_center.0 as i32 - candidate.center.0 as i32,
            old_center.1 as i32 - candidate.center.1 as i32,
        );
        for _ in 0..5 {
            if !there_is_collision(&candidate, obstacle_list.iter()) {
                return Some(candidate);
            }
            candidate.shift(0, 1);
        }
        None
    };

    if let Some((_, mut hold_t)) = take_controlled_by(hold.reborrow(), client) {
        // A piece is already held: swap it with the active piece.
        let Some(new_active) = try_place(*hold_t) else {
            return;
        };
        *active_t = new_active;
        *hold_t = new_hold;
    } else {
        // No piece held yet: park the active piece, promote the next piece, and
        // draw a fresh next piece.
        let Some((next_entity, mut next_t)) = take_controlled_by(next.reborrow(), client) else {
            return;
        };
        let Some(new_active) = try_place(*next_t) else {
            return;
        };
        let Some(mut private) = take_controlled_by(states.reborrow(), client) else {
            return;
        };

        *active_t = new_hold;
        commands
            .entity(active_entity)
            .remove::<Active>()
            .insert(Hold);

        *next_t = new_active;
        commands.entity(next_entity).remove::<Next>().insert(Active);

        private.bag.next_tetromino();
        let mut fresh_next = private.bag.peek();
        place_preview(&mut fresh_next);
        commands.spawn((
            fresh_next,
            Next,
            crate::controlled_by(client),
            replicate_to(client, &peer_ids),
        ));
    }
}

/// Update hard drop state based on user input
pub fn update_hard_drop(
    trigger: On<ReceivedInput>,
    states: Query<(&mut SharedGameState, &ControlledBy)>,
) {
    if !trigger.inputs.0.contains(&Input::HardDrop) {
        return;
    }
    let client = trigger.target;
    if let Some(mut state) = take_controlled_by(states, client) {
        state.hard_drop = !state.hard_drop;
        state.manual_drop_gravity = if state.hard_drop {
            HARD_DROP_GRAVITY
        } else {
            SOFT_DROP_GRAVITY
        };
    }
}
