use std::{cell::RefCell, rc::Rc};

use crate::{
    proto::Event,
    rpc_connection::ServerRpcConnection,
    server::{
        peer::PeerConnection,
        snapshot_resolver::{MemberId, SnapshotResolver},
    },
};

pub struct Room {
    #[allow(dead_code)]
    peers: Vec<PeerConnection>,
    snapshot_resolver: Option<SnapshotResolver>,
}

impl Room {
    pub fn new() -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(Self {
            peers: Vec::new(),
            snapshot_resolver: None,
        }));

        let snapshot_resolver = SnapshotResolver::new(Rc::downgrade(&this));
        this.borrow_mut().snapshot_resolver = Some(snapshot_resolver);

        this
    }

    pub fn on_make_sdp_offer(&mut self, _peer_id: u64, _sdp_offer: String) {
        self.snapshot_resolver.as_ref().unwrap().send_event(
            1,
            Event::SdpAnswerMade {
                peer_id: 1,
                sdp_answer: "Hello world!".to_string(),
            },
        );
        self.snapshot_resolver.as_ref().unwrap().send_event(
            2,
            Event::SdpAnswerMade {
                peer_id: 1,
                sdp_answer: "Hello world!".to_string(),
            },
        );
    }

    pub fn new_member_conn(
        &self,
        member_id: MemberId,
        conn: Box<dyn ServerRpcConnection>,
    ) {
        self.snapshot_resolver
            .as_ref()
            .unwrap()
            .new_member_conn(member_id, conn);
    }
}
