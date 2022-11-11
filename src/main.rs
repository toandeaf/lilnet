mod client;
mod server;

use client::is_anyone_home;

use std::{collections::BTreeSet, sync::Mutex};

#[macro_use]
extern crate lazy_static;

use local_ip_address::local_ip;
use reqwest::Client;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use crate::server::process_request;

lazy_static! {
    static ref GLOBAL_DATA: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    static ref CLIENT: Client = Client::new();
}

enum LilnetAction {
    NewJoin,
    Placeholder
}
struct LilnetRequest {
    action: LilnetAction,
    body: Option<String>
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let my_local_ip = local_ip().unwrap();

    is_anyone_home(my_local_ip, 100, 6969).await;

    let listener = TcpListener::bind("0.0.0.0:6969").await?;

    println!("Starting server...");

    loop {

        let (socket, socket_addr) = listener.accept().await?;

        tokio::spawn(async move { process_request(socket, socket_addr) });
    }
}
