use crate::{LilnetAction, LilnetRequest, GLOBAL_DATA};
use reqwest::Error;
use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const MESSAGE_SIZE: usize = 1024;

pub async fn process_request(socket: TcpStream, socket_addr: SocketAddr) -> Result<String, Error> {
    let request = parse_request(socket, socket_addr).await;

    match request.action {
        LilnetAction::NewJoin => {
            add_to_own_list(socket_addr.ip().to_string());
            Ok(String::from("Done the new Join!"))
        }
        LilnetAction::List => {
            dump_list();
            Ok(String::from("Done the List!"))
        }
    }
}

pub async fn parse_request(mut socket: TcpStream, socket_addr: SocketAddr) -> LilnetRequest {
    println!("Got a connection!");

    let mut buf = vec![0; MESSAGE_SIZE];

    loop {
        let n = socket.read(&mut buf).await;
        if n.unwrap() < MESSAGE_SIZE {
            break;
        }
    }

    let _ = &socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await;

    let parsed_string = match String::from_utf8(buf.to_vec()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let network_location = parsed_string
        .split('\n')
        .map(String::from)
        .find(|line| line.contains("HTTP/1.1"));

    let request_body = Some(socket_addr.ip().to_string());

    let action = match network_location {
        Some(val) if val.contains("/list") => LilnetAction::List,
        Some(_val) => LilnetAction::NewJoin,
        None => LilnetAction::NewJoin,
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

fn remove_from_list(address: String) {
    GLOBAL_DATA
        .lock()
        .map(|mut data| data.remove(address.as_str()))
        .expect("TODO Panic message");
}

pub async fn process_leaver(absent_address: String) {
    remove_from_list(absent_address)
}
