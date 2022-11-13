use crate::{LilnetAction, LilnetRequest, GLOBAL_DATA};
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct LilNetServer {}

impl LilNetServer {
    pub async fn initialise() -> JoinHandle<()> {
        println!("Server activity initiating...");
        let server_thread: JoinHandle<()> = tokio::spawn(async {
            kick_off().await;
        });
        println!("Server activity initiated...");

        server_thread
    }
}

async fn kick_off() {
    let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();

    loop {
        let results = listener.accept().await;
        if let Ok((socket, socket_addr)) = results {
            process_request(socket, socket_addr).await;
        }
    }
}

const MESSAGE_SIZE: usize = 1024;

pub async fn process_request(socket: TcpStream, socket_addr: SocketAddr) {
    let request = parse_request(socket, socket_addr).await;
    let address = socket_addr.ip().to_string();
    match request.action {
        LilnetAction::Ping => {
            let message = format!("Processing the Ping from {}!", &address);
            add_to_own_list(address);
            println!("{}", message);
        }
        LilnetAction::Ack => {
            let message = format!("Processing the Ack from {}!", &address);
            add_to_own_list(address);
            println!("{}", message);
        }
        LilnetAction::List => {
            dump_list();
        }
    }
}

pub async fn parse_request(mut socket: TcpStream, socket_addr: SocketAddr) -> LilnetRequest {
    let mut buf = vec![0; MESSAGE_SIZE];

    loop {
        let n = socket.read(&mut buf).await;
        if n.unwrap() < MESSAGE_SIZE {
            break;
        }
    }

    let _ = &socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await;

    // TODO Re-evaluate whether this is processing shit correctly or not.
    socket
        .flush()
        .await
        .map(|_| println!("Flushed!"))
        .expect("Failed to flush!");

    socket
        .shutdown()
        .await
        .map(|_| println!("Shutdown socket!"))
        .expect("Failed to shutdown!");

    let parsed_string = match String::from_utf8(buf.to_vec()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let network_location = parsed_string
        .split('\n')
        .map(String::from)
        .find(|line| line.contains("HTTP/1.1"));

    // TODO Make this useful
    let request_body = Some(socket_addr.ip().to_string());

    let action = match network_location {
        Some(val) if val.contains("/ping") => LilnetAction::Ping,
        Some(val) if val.contains("/ack") => LilnetAction::Ack,
        Some(val) if val.contains("/list") => LilnetAction::List,
        Some(_val) => LilnetAction::Ack,
        None => LilnetAction::Ack,
    };

    LilnetRequest {
        action,
        body: request_body,
    }
}

fn add_to_own_list(address: String) {
    GLOBAL_DATA
        .lock()
        .map(|mut data| data.insert(address))
        .expect("TODO: panic message");
}

fn dump_list() {
    GLOBAL_DATA
        .lock()
        .map(|data| {
            println!("Dumping contents");
            for dat in data.iter() {
                println!("{:?}", dat);
            }
        })
        .expect("Failed list dump");
}
