//! Scoring subsystem

use super::data::*;
use crate::*;
use bevy::prelude::*;
use lightyear::prelude::ControlledBy;

/// An event denoting that some lines are cleared
#[derive(EntityEvent)]
#[allow(missing_docs)]
pub struct LinesCleared {
    pub lines_cleared: u32,
    #[event_target]
    pub target: Entity,
}

/// Update the game state when lines are cleared
pub fn update_score(event: On<LinesCleared>, states: Query<(&mut SharedGameState, &ControlledBy)>) {
    let Some(mut state) = take_controlled_by(states, event.target) else {
        return;
    };
    let multiplier = state.level + 1;
    let points = match event.lines_cleared {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => 0,
    };
    state.score += points * multiplier;
    state.lines_cleared += event.lines_cleared;
    state.lines_cleared_since_last_level += event.lines_cleared;
    while state.lines_cleared_since_last_level >= 10 {
        state.level += 1;
        state.lines_cleared_since_last_level -= 10;
    }
}
