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
pub fn update_score(
    _event: On<LinesCleared>,
    _states: Query<(&mut SharedGameState, &ControlledBy)>,
) {
    todo!()
}
