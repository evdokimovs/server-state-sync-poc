use crate::{
    rpc_connection::ServerRpcConnection,
    server::{room::Room, snapshot_resolver::SnapshotResolver},
    snapshot::RoomSnapshot,
};

mod peer;
mod room;
pub mod snapshot_resolver;
mod track;

pub struct Server {
    room: SnapshotResolver,
}

impl Server {
    pub fn new(resolver: SnapshotResolver) -> Self {
        Self { room: resolver }
    }
}
