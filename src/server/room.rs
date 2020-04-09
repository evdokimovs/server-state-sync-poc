use crate::{
    proto::Event,
    server::{
        peer::PeerConnection,
        snapshot_resolver::{EventResponse, EventSender, SnapshotResolver},
    },
    snapshot::RoomSnapshot,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Room {
    peers: Vec<PeerConnection>,
}

impl Room {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn on_make_sdp_offer<R>(
        &mut self,
        mut snapshot_resolver: R,
        peer_id: u64,
        sdp_offer: String,
    ) where
        R: EventSender,
    {
        snapshot_resolver.send_event(
            1,
            Event::SdpAnswerMade {
                peer_id: 1,
                sdp_answer: "hello".to_string(),
            },
        );
        snapshot_resolver.send_event(
            2,
            Event::SdpAnswerMade {
                peer_id: 1,
                sdp_answer: "hello".to_string(),
            },
        );
    }
}
