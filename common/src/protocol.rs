//! Our application-level communication protocol

use std::collections::HashSet;

use bevy::prelude::*;
use lightyear::connection::prelude::NetworkDirection;
use lightyear::prelude::{
    AppChannelExt, AppComponentExt, AppMessageExt, ChannelMode, ChannelSettings, ReliableSettings,
};
use serde::{Deserialize, Serialize};

use crate::board::Block;
use crate::data::{Active, Hold, Next, Obstacle, SharedGameState, Tetromino};

/// Channel for keyboard inputs to send over the network.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct InputChannel;

/// Inputs from the client
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Input {
    Left,
    Right,
    Down,
    Rotate,
    Hold,
    HardDrop,
    EndGame,
}

/// Set of logical inputs to be sent as a message
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Default, Debug)]
pub struct Inputs(pub HashSet<Input>);

/// Heartbeat message and channel
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Reflect)]
pub struct Heartbeat;

/// Protocol plugin added to both server and client
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<SharedGameState>();
        app.register_component::<Tetromino>();
        app.register_component::<Block>();
        app.register_component::<Active>();
        app.register_component::<Next>();
        app.register_component::<Obstacle>();
        app.register_component::<Hold>();

        app.add_channel::<InputChannel>(ChannelSettings {
            mode: ChannelMode::SequencedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.add_channel::<Heartbeat>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<Inputs>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<Heartbeat>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}
