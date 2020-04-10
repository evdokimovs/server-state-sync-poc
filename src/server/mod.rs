use std::{cell::RefCell, rc::Rc};

use crate::{
    rpc_connection::ServerRpcConnection,
    server::{room::Room, snapshot_resolver::SnapshotResolver},
    snapshot::RoomSnapshot,
};

mod peer;
pub mod room;
pub mod snapshot_resolver;
mod track;

pub struct Server {
    room: Rc<RefCell<Room>>,
}

impl Server {
    pub fn new(room: Rc<RefCell<Room>>) -> Self {
        Self { room }
    }
}
