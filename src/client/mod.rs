use tokio::task::spawn_local;

use crate::{
    client::room::Room,
    proto::Command,
    rpc_connection::ClientRpcConnection,
    snapshot::{PeerSnapshot, RoomSnapshot},
};
use futures::StreamExt;

mod peer;
mod room;
mod track;

fn test_room_snapshot() -> RoomSnapshot {
    let mut room = RoomSnapshot::new();
    let mut peer = PeerSnapshot {
        sdp_offer: Some("hello".to_string()),
        sdp_answer: None,
    };
    room.insert_peer(1, peer);

    room
}

pub struct Client {
    room: Room,
    rpc: Box<dyn ClientRpcConnection>,
}

impl Client {
    pub fn new(id: u64, rpc: Box<dyn ClientRpcConnection>) -> Self {
        let mut events_stream = rpc.on_event();
        spawn_local(async move {
            while let Some(event) = events_stream.next().await {
                println!("MemberId: {}; Event: {:?}", id, event);
            }
        });

        Self {
            room: Room::new(),
            rpc,
        }
    }

    pub fn reconnect(&self) {
        let room_snapshot = test_room_snapshot();
        self.rpc.send_command(Command::SynchronizeMe {
            snapshot: room_snapshot,
        });
    }
}
