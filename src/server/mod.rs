mod peer;
mod room;
mod snapshot_resolver;
mod track;

use std::{cell::RefCell, rc::Rc};

pub use self::{room::Room, snapshot_resolver::SnapshotResolver};

pub struct Server {
    #[allow(dead_code)]
    room: Rc<RefCell<Room>>,
}

impl Server {
    pub fn new(room: Rc<RefCell<Room>>) -> Self {
        Self { room }
    }
}
