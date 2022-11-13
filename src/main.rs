mod client;
mod server;

use client::is_anyone_home;
use local_ip_address::local_ip;
use reqwest::{Client, Error};

use crate::client::client_iteration;
use crate::server::process_request;
use std::thread::ThreadId;
use std::{collections::BTreeSet, sync::Mutex};
use tokio::net::TcpListener;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GLOBAL_DATA: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    static ref CLIENT: Client = Client::new();
}

enum LilnetAction {
    Ping,
    Ack,
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

    let my_local_ip = local_ip().unwrap();
    is_anyone_home(my_local_ip, 100, 6969).await;

    // TODO Implement thread pool
    tokio::spawn(async {
        loop {
            client_iteration().await;
        }
    });
    println!("Client activity initiated...");

    loop {
        let outcome = match listener.accept().await {
            Ok((socket, socket_addr)) => {
                let handler = tokio::spawn(async move {
                    let outcome: Result<String, Error> = process_request(socket, socket_addr).await;
                    println!("{}", outcome.unwrap());
                    Ok(())
                });
                handler.await.unwrap()
            }
            Err(e) => {
                eprintln!("Error handling connection!");
                Err(e)
            }
        };

        match outcome {
            Ok(_) => println!("Connection handled, returning thread to pool..."),
            Err(_) => eprintln!("Issue handling connection!"),
        }
    }
}
