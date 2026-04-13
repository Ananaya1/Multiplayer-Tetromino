//! The networking code, the bulk of the server.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use common::config::GameConfig;
use common::protocol::*;
use common::*;
use core::net::SocketAddr;
use core::time::Duration;
use lightyear::core::tick::TickDuration;
use lightyear::core::time::TickInstant;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::net::{Ipv4Addr, Ipv6Addr};

/// The last time we received a heartbeat from a client.
#[derive(Component)]
pub struct LastReceived(Duration);

// A helper to convert lightyear ticks to durations.  Not fully necessary.
fn current_time(timeline: &LocalTimeline, tick_duration: &TickDuration) -> Duration {
    TickInstant::from(timeline.tick()).as_duration(**tick_duration)
}

/// Inject the actual game server logic
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_observer(handle_new_client)
            .add_observer(handle_connected)
            .add_systems(
                Update,
                (
                    handle_input,
                    handle_heartbeat,
                    debug_server.run_if(on_timer(Duration::from_millis(1000))),
                    disconnect_unresponsive_clients.run_if(on_timer(HEARTBEAT_INTERVAL)),
                ),
            );
        crate::build_app(app);
    }
}

/// Spawn the WebSocket server entity and begin listening.
fn startup(mut commands: Commands, cfg: Res<GameConfig>) {
    let server_addr = if cfg.server_addr.is_ipv4() {
        Ipv4Addr::UNSPECIFIED.into()
    } else {
        Ipv6Addr::UNSPECIFIED.into()
    };
    let bind_addr = SocketAddr::new(server_addr, cfg.server_port);
    let ws_config = ServerConfig::builder()
        .with_bind_default(cfg.server_port)
        .with_no_encryption();
    let server = commands
        .spawn((
            RawServer,
            WebSocketServerIo { config: ws_config },
            LocalAddr(bind_addr),
        ))
        .id();
    commands.trigger(Start { entity: server });
    info!("Starting server on {bind_addr}");
}

/// Periodically log the server state
pub fn debug_server(servers: Query<(Entity, &Server)>) {
    for (entity, server) in servers {
        info!("server {:?}: {:?}", entity, server);
    }
}

/// When a new client link is established, enable replication to that client.
pub fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands, cfg: Res<GameConfig>) {
    info!("new client for {:?}", trigger.entity);
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            cfg.replication_interval(),
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

/// When a client is fully connected, spawn the block entity and replicate it.
///
/// Each client gets its own block.
pub fn handle_connected(
    trigger: On<Add, Connected>,
    peer_ids: Query<&RemoteId, With<ClientOf>>,
    _cfg: Res<GameConfig>,
    _commands: Commands,
) {
    let client_id = peer_ids
        .get(trigger.entity)
        .expect("Possible networking stack bug: Client connected with no remote ID!");
    info!("Client {:?} ({:?}) connected", trigger.entity, client_id);

    todo!("spawn global state related to this client, replicate the parts that make sense.")
}

/// Apply received Direction messages to the block position each frame.
/// All receivers are drained, but movement is only applied for the block owner.
pub fn handle_input(
    _receivers: Query<(Entity, &mut MessageReceiver<Inputs>)>,
    _commands: Commands,
) {
    todo!("drain all receivers, trigger ReceivedInput for each relevant client.")
}

/// Refresh clients that send a heartbeat signal
pub fn handle_heartbeat(
    receivers: Query<(Entity, &mut MessageReceiver<Heartbeat>)>,
    mut commands: Commands,
    timeline: Res<LocalTimeline>,
    tick_duration: Res<TickDuration>,
) {
    for (client, mut receiver) in receivers {
        if receiver.receive().last().is_some() {
            info!("Heartbeat received from {client:?}");
            commands
                .entity(client)
                .insert(LastReceived(current_time(&timeline, &tick_duration)));
        }
    }
}

/// Disconnect clients that we haven't received a message from in a long time.
/// If the disconnected client controlled the block, transfer control to another
/// alive client, or despawn the block if none remain.
pub fn disconnect_unresponsive_clients(
    clients: Query<(Entity, &LastReceived)>,
    timeline: Res<LocalTimeline>,
    tick_duration: Res<TickDuration>,
    mut commands: Commands,
) {
    let current_time = current_time(&timeline, &tick_duration);

    for (entity, LastReceived(last_received_time)) in &clients {
        if current_time - *last_received_time > DISCONNECT_INTERVAL {
            warn!("{entity:?} has been stale! disconnecting.");
            commands.entity(entity).despawn();
        }
    }
}
