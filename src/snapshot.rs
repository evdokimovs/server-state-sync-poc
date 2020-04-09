use std::{cell::RefCell, rc::Rc};

use futures::Stream;
use medea_reactive::{collections::ObservableVec, Observable};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RoomSnapshot {
    pub peers: HashMap<u64, PeerSnapshot>,
}

impl RoomSnapshot {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn insert_peer(&mut self, peer_id: u64, peer: PeerSnapshot) {
        self.peers.insert(peer_id, peer);
    }
}

#[derive(Clone, Debug)]
pub struct PeerSnapshot {
    pub sdp_offer: Option<String>,
    pub sdp_answer: Option<String>,
}

pub struct PeerSnapshotDiff {
    pub sdp_offer_change: Option<Option<String>>,
    pub sdp_answer_change: Option<Option<String>>,
}

pub struct OnMakeSdpOfferPeerSnapshotDiffGroup {
    pub sdp_offer: Option<String>,
}

impl PeerSnapshotDiff {
    pub fn on_make_sdp_offer(
        &self,
    ) -> Option<OnMakeSdpOfferPeerSnapshotDiffGroup> {
        if let Some(sdp_offer) = &self.sdp_offer_change {
            return Some(OnMakeSdpOfferPeerSnapshotDiffGroup {
                sdp_offer: sdp_offer.clone(),
            });
        }

        None
    }
}

impl PeerSnapshot {
    pub fn diff(&self, another: &PeerSnapshot) -> PeerSnapshotDiff {
        let mut diff = PeerSnapshotDiff {
            sdp_answer_change: None,
            sdp_offer_change: None,
        };

        if self.sdp_answer != another.sdp_answer {
            diff.sdp_answer_change = Some(another.sdp_answer.clone());
        }
        if self.sdp_offer != another.sdp_offer {
            diff.sdp_offer_change = Some(another.sdp_offer.clone());
        }

        diff
    }
}

#[derive(Debug)]
pub struct TrackSnapshot {
    pub is_muted: bool,
}

impl TrackSnapshot {
    pub fn new(is_muted: bool) -> Self {
        Self { is_muted }
    }
}
