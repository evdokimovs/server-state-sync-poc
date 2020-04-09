use crate::server::track::Track;

#[derive(Debug)]
pub struct PeerConnection {
    tracks: Vec<Track>,
}
