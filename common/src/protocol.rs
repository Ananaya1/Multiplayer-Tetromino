//! Our application-level communication protocol

use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};


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
    fn build(&self, _app: &mut App) {
        // - Register components that should be replicated
        // - Create the channel InputChannel for inputs, should be sequenced
        // - Create the channel Heartbeat for heartbeat signals, should be unordered
        // - Register the messages we can send
        //
        // Both channels should be reliable.
        //
        // See the in-class example for how to do this.
    }
}
