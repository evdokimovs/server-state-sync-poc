use std::time::Duration;

use tokio::task::LocalSet;

use crate::{
    client::Client,
    rpc_connection::RpcConnection,
    server::{room::Room, snapshot_resolver::SnapshotResolver, Server},
};

mod client;
mod proto;
mod rpc_connection;
mod server;
mod snapshot;

#[tokio::main]
async fn main() {
    let local = LocalSet::new();

    local
        .run_until(async move {
            let room = Room::new();

            let alice_rpc = RpcConnection::new();
            let alice_client = Client::new(1, Box::new(alice_rpc.clone()));
            room.borrow_mut()
                .new_member_conn(1, Box::new(alice_rpc.clone()));

            let bob_rpc = RpcConnection::new();
            let bob_client = Client::new(2, Box::new(bob_rpc.clone()));
            room.borrow_mut()
                .new_member_conn(2, Box::new(bob_rpc.clone()));

            let server = Server::new(room);

            alice_client.reconnect();

            tokio::time::delay_for(Duration::from_secs(5)).await;
        })
        .await;
}
