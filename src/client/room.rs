use crate::client::peer::PeerConnection;

pub struct Room {
    peers: Vec<PeerConnection>,
}

impl Room {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }
}
