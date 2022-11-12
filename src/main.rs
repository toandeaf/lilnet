mod client;
mod server;

use client::is_anyone_home;
use local_ip_address::local_ip;
use reqwest::{Client, Error};

use crate::client::client_iteration;
use crate::server::process_request;
use std::{collections::BTreeSet, sync::Mutex, thread};
use tokio::net::TcpListener;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GLOBAL_DATA: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    static ref CLIENT: Client = Client::new();
}

enum LilnetAction {
    NewJoin,
    List,
}

pub struct LilnetRequest {
    action: LilnetAction,
    body: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    let listener = TcpListener::bind("0.0.0.0:6969").await?;
    println!("Listening for connections...");

    tokio::spawn(async {
        let my_local_ip = local_ip().unwrap();
        is_anyone_home(my_local_ip, 100, 6969).await;

        loop {
            client_iteration().await;
        }
    });

    println!("Client activity initiated...");

    loop {
        match listener.accept().await {
            Ok((socket, socket_addr)) => {
                tokio::spawn(async move {
                    let outcome: Result<String, Error> = process_request(socket, socket_addr).await;
                    println!("{}", outcome.unwrap());
                });
            }
            Err(_) => {
                eprintln!("Error handling connection!");
            }
        }
    }
}
