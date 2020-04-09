use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use futures::StreamExt as _;
use tokio::task::spawn_local;

use crate::{
    proto::{Command, Event},
    rpc_connection::ServerRpcConnection,
    server::room::Room,
    snapshot::{PeerSnapshot, RoomSnapshot},
};

type MemberId = u64;

pub struct EventSenderImpl<'a> {
    current_snapshot: &'a mut HashMap<MemberId, RoomSnapshot>,
    connections: &'a mut HashMap<MemberId, Box<dyn ServerRpcConnection>>,
    members_in_sync_state: &'a mut HashSet<MemberId>,
}

pub trait EventSender {
    fn send_event(&mut self, member_id: MemberId, event: Event);
}

impl<'a> EventSender for EventSenderImpl<'a> {
    fn send_event(&mut self, member_id: u64, event: Event) {
        if let Some(current_snapshot) =
            self.current_snapshot.get_mut(&member_id)
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
                    self.members_in_sync_state.remove(&member_id);
                }
            }

            if !self.members_in_sync_state.contains(&member_id) {
                if let Some(conn) = self.connections.get(&member_id) {
                    conn.send_event(event);
                }
            }
        }
    }
}

pub struct EventResponse(pub HashMap<MemberId, Event>);

struct SnapshotResolverInner {
    current_snapshot: HashMap<MemberId, RoomSnapshot>,
    room: Room,
    connections: HashMap<MemberId, Box<dyn ServerRpcConnection>>,
    members_in_sync_state: HashSet<MemberId>,
}

impl SnapshotResolverInner {
    pub fn new() -> Self {
        Self {
            current_snapshot: HashMap::new(),
            room: Room::new(),
            connections: HashMap::new(),
            members_in_sync_state: HashSet::new(),
        }
    }

    fn event_sender(&mut self) -> EventSenderImpl {
        EventSenderImpl {
            current_snapshot: &mut self.current_snapshot,
            connections: &mut self.connections,
            members_in_sync_state: &mut self.members_in_sync_state,
        }
    }

    fn sync(&mut self, member_id: MemberId, with_snapshot: RoomSnapshot) {
        let peer_diffs: Vec<_> = if let Some(room_snapshot) =
            self.current_snapshot.get_mut(&member_id)
        {
            self.members_in_sync_state.insert(member_id);
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
                let room = &mut self.room;
                let sender = EventSenderImpl {
                    connections: &mut self.connections,
                    current_snapshot: &mut self.current_snapshot,
                    members_in_sync_state: &mut self.members_in_sync_state,
                };
                room.on_make_sdp_offer(
                    sender,
                    peer_id,
                    on_make_sdp_offer.sdp_offer.unwrap(),
                );
            }
        }

        let final_room_snapshot =
            self.current_snapshot.get(&member_id).unwrap().clone();
        self.event_sender().send_event(
            member_id,
            Event::SnapshotSynchronized {
                snapshot: final_room_snapshot,
            },
        );
    }

    fn on_command(&mut self, member_id: MemberId, command: Command) {
        match command {
            Command::MakeSdpAnswer {
                peer_id,
                sdp_answer,
            } => {
                let mut room = &mut self.room;
                let sender = EventSenderImpl {
                    connections: &mut self.connections,
                    current_snapshot: &mut self.current_snapshot,
                    members_in_sync_state: &mut self.members_in_sync_state,
                };
                room.on_make_sdp_offer(sender, peer_id, sdp_answer.clone());
            }
            Command::SynchronizeMe { snapshot } => {
                self.sync(member_id, snapshot);
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotResolver(Rc<RefCell<SnapshotResolverInner>>);

impl SnapshotResolver {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(SnapshotResolverInner::new())))
    }

    pub fn new_member_conn(
        &self,
        member_id: MemberId,
        conn: Box<dyn ServerRpcConnection>,
    ) {
        let mut command_stream = conn.on_command();
        let this = self.clone();
        let mut inner = self.0.borrow_mut();
        inner.connections.insert(member_id, conn);

        let mut room_snap = RoomSnapshot::new();
        room_snap.peers.insert(
            1,
            PeerSnapshot {
                sdp_answer: None,
                sdp_offer: None,
            },
        );
        inner.current_snapshot.insert(member_id.clone(), room_snap);

        spawn_local(async move {
            while let Some(command) = command_stream.next().await {
                this.0.borrow_mut().on_command(member_id.clone(), command);
            }
        });
    }
}
