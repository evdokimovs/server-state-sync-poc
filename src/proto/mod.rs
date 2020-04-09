use crate::snapshot::RoomSnapshot;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Command {
    MakeSdpOffer {
        peer_id: u64,
        sdp_offer: String,
    },

    MakeSdpAnswer {
        peer_id: u64,
        sdp_answer: String,
    },

    UpdateTrack {
        peer_id: u64,
        track_id: u64,
        is_muted: bool,
    },

    SynchronizeMe {
        snapshot: RoomSnapshot,
    },
}

#[derive(Debug)]
pub enum Event {
    SnapshotSynchronized { snapshot: RoomSnapshot },

    SdpAnswerMade { peer_id: u64, sdp_answer: String },

    PeersRemoved { peers_ids: Vec<u64> },
}
