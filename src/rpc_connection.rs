use std::{cell::RefCell, pin::Pin, rc::Rc};

use futures::{channel::mpsc, Stream};

use crate::proto::{Command, Event};

struct InnerRpcConnection {
    event_sender: Option<mpsc::UnboundedSender<Event>>,
    command_sender: Option<mpsc::UnboundedSender<Command>>,
}

#[derive(Clone)]
pub struct RpcConnection(Rc<RefCell<InnerRpcConnection>>);

impl RpcConnection {
    pub fn new() -> Self {
        let inner = InnerRpcConnection {
            event_sender: None,
            command_sender: None,
        };

        Self(Rc::new(RefCell::new(inner)))
    }
}

pub trait ClientRpcConnection {
    fn on_event(&self) -> Pin<Box<dyn Stream<Item = Event>>>;

    fn send_command(&self, command: Command);
}

pub trait ServerRpcConnection {
    fn on_command(&self) -> Pin<Box<dyn Stream<Item = Command>>>;

    fn send_event(&self, event: Event);
}

impl ClientRpcConnection for RpcConnection {
    fn on_event(&self) -> Pin<Box<dyn Stream<Item = Event>>> {
        let (tx, rx) = mpsc::unbounded();

        self.0.borrow_mut().event_sender = Some(tx);

        Box::pin(rx)
    }

    fn send_command(&self, command: Command) {
        if let Some(command_sender) = &self.0.borrow().command_sender {
            let _ = command_sender.unbounded_send(command);
        }
    }
}

impl ServerRpcConnection for RpcConnection {
    fn on_command(&self) -> Pin<Box<dyn Stream<Item = Command>>> {
        let (tx, rx) = mpsc::unbounded();

        self.0.borrow_mut().command_sender = Some(tx);

        Box::pin(rx)
    }

    fn send_event(&self, event: Event) {
        if let Some(event_sender) = &self.0.borrow().event_sender {
            let _ = event_sender.unbounded_send(event);
        }
    }
}
