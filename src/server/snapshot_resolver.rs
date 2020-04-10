use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};

use futures::StreamExt as _;
use tokio::task::spawn_local;

use crate::{
    proto::{Command, Event},
    rpc_connection::ServerRpcConnection,
    server::room::Room,
    snapshot::{PeerSnapshot, RoomSnapshot},
};

pub(crate) type MemberId = u64;

struct SnapshotResolverInner {
    current_snapshot: RefCell<HashMap<MemberId, RoomSnapshot>>,
    room: Weak<RefCell<Room>>,
    connections: RefCell<HashMap<MemberId, Box<dyn ServerRpcConnection>>>,
    members_in_sync_state: RefCell<HashSet<MemberId>>,
}

impl SnapshotResolverInner {
    pub fn new(room: Weak<RefCell<Room>>) -> Self {
        Self {
            current_snapshot: RefCell::new(HashMap::new()),
            room,
            connections: RefCell::new(HashMap::new()),
            members_in_sync_state: RefCell::new(HashSet::new()),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotResolver(Rc<SnapshotResolverInner>);

impl SnapshotResolver {
    pub fn new(room: Weak<RefCell<Room>>) -> Self {
        Self(Rc::new(SnapshotResolverInner::new(room)))
    }

    pub fn new_member_conn(
        &self,
        member_id: MemberId,
        conn: Box<dyn ServerRpcConnection>,
    ) {
        let mut command_stream = conn.on_command();
        self.0.connections.borrow_mut().insert(member_id, conn);

        let mut room_snap = RoomSnapshot::new();
        room_snap.peers.insert(
            1,
            PeerSnapshot {
                sdp_answer: None,
                sdp_offer: None,
            },
        );
        self.0
            .current_snapshot
            .borrow_mut()
            .insert(member_id.clone(), room_snap);

        let weak_inner = Rc::downgrade(&self.0);
        spawn_local(async move {
            while let Some(command) = command_stream.next().await {
                if let Some(inner) = weak_inner.upgrade() {
                    let this = SnapshotResolver(inner);
                    this.on_command(member_id.clone(), command);
                }
            }
        });
    }

    fn on_command(&self, member_id: MemberId, command: Command) {
        match command {
            Command::MakeSdpAnswer {
                peer_id,
                sdp_answer,
            } => {
                if let Some(room) = self.0.room.upgrade() {
                    room.borrow_mut()
                        .on_make_sdp_offer(peer_id, sdp_answer.clone());
                }
            }
            Command::SynchronizeMe { snapshot } => {
                self.sync(member_id, snapshot);
            }
            _ => (),
        }
    }

    pub fn send_event(&self, member_id: u64, event: Event) {
        if let Some(current_snapshot) =
            self.0.current_snapshot.borrow_mut().get_mut(&member_id)
        {
            match &event {
                Event::PeersRemoved { peers_ids } => {
                    for peer_id in peers_ids {
                        current_snapshot.peers.remove(&peer_id);
                    }
                }
                Event::SdpAnswerMade {
                    peer_id,
                    sdp_answer,
                } => {
                    if let Some(peer) = current_snapshot.peers.get_mut(&peer_id)
                    {
                        peer.sdp_answer = Some(sdp_answer.clone());
                    }
                }
                Event::SnapshotSynchronized { snapshot } => {
                    self.0
                        .members_in_sync_state
                        .borrow_mut()
                        .remove(&member_id);
                }
            }

            if !self.0.members_in_sync_state.borrow().contains(&member_id) {
                if let Some(conn) = self.0.connections.borrow().get(&member_id)
                {
                    conn.send_event(event);
                }
            }
        }
    }

    fn sync(&self, member_id: MemberId, with_snapshot: RoomSnapshot) {
        let peer_diffs: Vec<_> = if let Some(room_snapshot) =
            self.0.current_snapshot.borrow().get(&member_id)
        {
            self.0.members_in_sync_state.borrow_mut().insert(member_id);
            with_snapshot
                .peers
                .into_iter()
                .filter_map(|(peer_id, new_peer)| {
                    room_snapshot.peers.get(&peer_id).map(|current_peer| {
                        (peer_id, current_peer.diff(&new_peer))
                    })
                })
                .collect()
        } else {
            return;
        };

        for (peer_id, peer_diff) in peer_diffs {
            if let Some(on_make_sdp_offer) = peer_diff.on_make_sdp_offer() {
                if let Some(room) = self.0.room.upgrade() {
                    room.borrow_mut().on_make_sdp_offer(
                        peer_id,
                        on_make_sdp_offer.sdp_offer.unwrap(),
                    );
                }
            }
        }

        let final_room_snapshot = self
            .0
            .current_snapshot
            .borrow()
            .get(&member_id)
            .unwrap()
            .clone();
        self.send_event(
            member_id,
            Event::SnapshotSynchronized {
                snapshot: final_room_snapshot,
            },
        );
    }
}
