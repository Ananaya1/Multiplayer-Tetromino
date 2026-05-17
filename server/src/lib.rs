//! Server support library.

// these two flags are allowed because these warnings are triggered by many
// systems.
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::system::Query;

use common::*;
use lightyear::prelude::{ControlledBy, Lifetime, NetworkTarget, RemoteId, Replicate};

use crate::score::update_score;

pub mod game_logic;
pub mod net;
pub mod score;
#[cfg(test)]
pub mod tests;

/// Inject the systems and plugins for this game into the app.
///
/// This is the client version, so it should include only the input and
/// rendering logic.
pub fn build_app(app: &mut App) {
    use bevy::prelude::*;
    use board::*;
    use game_logic::*;
    app.add_systems(
        FixedUpdate,
        (
            gravity,
            deactivate_if_stuck,
            delete_full_lines,
            spawn_next_tetromino,
        )
            .chain()
            .in_set(Game),
    )
    .add_observer(handle_user_input)
    .add_observer(swap_hold)
    .add_observer(update_score)
    .add_observer(update_hard_drop)
    .add_observer(game_over_on_esc)
    .add_observer(disconnect_on_game_over);
}

/// Convert given query to a hash map indexed by the owner of each query data.
/// Assume that there is at most one entity per owner.
pub fn build_per_client_table<'a, C: QueryData, F: QueryFilter>(
    query: Query<'a, 'a, (C, &ControlledBy), F>,
) -> HashMap<Entity, C::Item<'a, 'a>> {
    query
        .into_iter()
        .map(|(item, controlled_by)| (controlled_by.owner, item))
        .collect()
}

/// Convert given query to a hash map indexed by the owner of each query data item.
pub fn build_per_client_lists<'a, C: QueryData, F: QueryFilter>(
    query: Query<'a, 'a, (C, &ControlledBy), F>,
) -> HashMap<Entity, Vec<C::Item<'a, 'a>>> {
    let mut out = HashMap::new();
    for (item, controlled_by) in query {
        out.entry(controlled_by.owner)
            .or_insert_with(Vec::new)
            .push(item);
    }
    out
}

/// Take out the query elements controlled by given entity.
pub fn filter_controlled_by<'a, C: QueryData, F: QueryFilter>(
    query: Query<'a, 'a, (C, &ControlledBy), F>,
    owner: Entity,
) -> impl Iterator<Item = C::Item<'a, 'a>> {
    query
        .into_iter()
        .filter(move |(_, controlled_by)| controlled_by.owner == owner)
        .map(|(item, _)| item)
}

/// Take out the single query element controlled by given entity.
pub fn take_controlled_by<'a, C: QueryData, F: QueryFilter>(
    query: Query<'a, 'a, (C, &ControlledBy), F>,
    owner: Entity,
) -> Option<C::Item<'a, 'a>> {
    filter_controlled_by(query, owner).next()
}

/// Create the owner info for the client to inject into an entity
pub fn controlled_by(client: Entity) -> ControlledBy {
    ControlledBy {
        owner: client,
        lifetime: Lifetime::SessionBased,
    }
}

/// Create replication info for replicating to a single client
pub fn replicate_to<F: QueryFilter>(client: Entity, peer_ids: &Query<&RemoteId, F>) -> Replicate {
    let peer_id = **peer_ids
        .get(client)
        .expect("client entity must have a RemoteId for targeted replication");
    Replicate::to_clients(NetworkTarget::Single(peer_id))
}
