use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
    sync::Mutex,
    time::Duration,
};

#[macro_use]
extern crate lazy_static;

use local_ip_address::local_ip;
use reqwest::Client;

use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio::{io::AsyncWriteExt, net::TcpStream};

lazy_static! {
    static ref GLOBAL_DATA: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let my_local_ip = local_ip().unwrap();

    is_anyone_home(my_local_ip, 100, 6969).await;

    let listener = TcpListener::bind("0.0.0.0:6969").await?;

    println!("Starting server...");

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move { process(socket).await });
    }
}

async fn process(mut socket: TcpStream) {
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

    // println!("result: {}", s); // Write the data back

    let network_lines = parsed_string
        .split("\n")
        .into_iter()
        .map(|val| String::from(val));

    let request_body = network_lines.last();

    match request_body {
        Some(val) => {
            println!("Request body is: {}", val);
            GLOBAL_DATA.lock().unwrap().insert(val);
        }
        None => println!("No request body"),
    }

    if let Err(e) = socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await {
        eprintln!("failed to write to socket; err = {:?}", e);
        return;
    }

    ()
}

async fn is_anyone_home(own_ip: IpAddr, max_range: u8, port: u32) {
    let ip_range = 1..max_range;

    let mut futs = vec![];

    for ip in ip_range {
        futs.push(async move {
            let address = Ipv4Addr::new(192, 168, 0, ip);
            let formatted_address = format!("http://{}:{}", address.to_string(), port.to_string());
            let request_body = format!("{own_ip}");

            dispatch_request(formatted_address, request_body).await
        });
    }

    futures::future::join_all(futs).await;
}

async fn dispatch_request(address: String, request_body: String) {
    let client = Client::new();

    let response = client
        .post(&address)
        .timeout(Duration::from_secs(2))
        .body(request_body)
        .header("Content-Type", "text/plain")
        .send()
        .await;

    match response {
        Ok(_result) => add_to_list(address),
        Err(_) => (),
    }
}

fn add_to_list(address: String) {
    GLOBAL_DATA.lock().unwrap().insert(address);
}
