use futures::StreamExt;
use tokio::task::spawn_local;

use crate::{
    proto::Command,
    rpc_connection::ClientRpcConnection,
    snapshot::{PeerSnapshot, RoomSnapshot},
};

fn test_room_snapshot() -> RoomSnapshot {
    let mut room = RoomSnapshot::new();
    let peer = PeerSnapshot {
        sdp_offer: Some("hello".to_string()),
        sdp_answer: None,
    };
    room.insert_peer(1, peer);

    room
}

pub struct Client {
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

        Self { rpc }
    }

    pub fn reconnect(&self) {
        let room_snapshot = test_room_snapshot();
        self.rpc.send_command(Command::SynchronizeMe {
            snapshot: room_snapshot,
        });
    }
}
