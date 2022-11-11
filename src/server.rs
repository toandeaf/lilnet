use std::io::Split;
use std::net::SocketAddr;
use crate::{GLOBAL_DATA, LilnetRequest, LilnetAction};

use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::TcpStream,
};


pub async fn process_request(socket: TcpStream, socket_addr: SocketAddr) {
    let request = parse_request(socket, socket_addr).await;

    match request.action {
        LilnetAction::NewJoin => {
            println!("{}", socket_addr);
            request.body.map(|body| add_to_own_list(body));
        }
        LilnetAction::Placeholder => {}
    }
}

pub async fn process_leaver(absent_address: String) {
    remove_from_list(absent_address)
}

async fn process_new_joiner(socket: TcpStream, socket_addr: SocketAddr) {
    let request = parse_request(socket, socket_addr).await;

    let request_body = request.body;

    match request_body {
        Some(address) => {
            println!("Request body is: {}", address);
            add_to_own_list(address);
        }
        None => println!("No request body"),
    }

    ()
}

fn add_to_own_list(address: String) {
    GLOBAL_DATA.lock().unwrap().insert(address);
}

fn remove_from_list(address: String) {
    GLOBAL_DATA.lock().unwrap().remove(address.as_str());
}

pub async fn parse_request(mut socket: TcpStream, socket_addr: SocketAddr) -> LilnetRequest {
    println!("Got a connection!");

    let mut buf = vec![0; 126];

    loop {
        let n = socket.read(&mut buf).await;
        if n.unwrap() == 0 {
            break;
        }
    }

    let parsed_string = match String::from_utf8(buf.to_vec()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let mut network_lines: Vec<String> = parsed_string
        .split("\n")
        .map(|line| String::from(line))
        .collect();

    for line in &network_lines {
        println!("Line: {}", &line);
    }

    let request_body = network_lines.pop();

    let request_loc = network_lines.get(0)
        .map(|val| val.split(" "))
        .map(| split_str| String::from(split_str.skip(1).next().unwrap()))
        .unwrap();

    println!("{:?}", socket_addr);


    if let Err(e) = &socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await {
        eprintln!("failed to write to socket; err = {:?}", e);
    }

    println!("{}", &request_body.clone().unwrap());

    return LilnetRequest {
        action: LilnetAction::NewJoin,
        body: request_body,
    };
}
