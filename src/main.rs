mod init;
mod wait;
use init::is_anyone_home;
use wait::process_new_joiner;

use std::{collections::BTreeSet, sync::Mutex};

#[macro_use]
extern crate lazy_static;

use local_ip_address::local_ip;
use reqwest::Client;

use tokio::{net::{TcpListener, TcpStream}, io::AsyncReadExt};

lazy_static! {
    static ref GLOBAL_DATA: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    static ref CLIENT: Client = Client::new();
}

enum LilnetAction {
    NewJoin,
    Ping
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
        let (socket, _) = listener.accept().await?;

        // let server_action: ServerAction = determine_server_action()
        tokio::spawn(async move { process_new_joiner(socket).await });
    }
}

pub async fn parse_response(socket: TcpStream) -> LilnetRequest {
    println!("Got a connection!");

    let mut buf = [0; 1024];

    match socket.read(&mut buf).await {
        // socket closed
        Ok(n) if n == 0 => return,
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e);
            return;
        }
    };

    let parsed_string = match String::from_utf8(buf.to_vec()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let network_lines = parsed_string
        .split("\n")
        .into_iter()
        .map(|val| String::from(val));

    let request_body = network_lines.last();

    return LilnetRequest {
        action: LilnetAction::NewJoin,
        body: request_body
    }

}
