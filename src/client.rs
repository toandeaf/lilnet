use crate::GLOBAL_DATA;
use local_ip_address::local_ip;
use reqwest::Client;
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
    thread,
    time::Duration,
};
use tokio::task::JoinHandle;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub struct LilNetClient {}

impl LilNetClient {
    pub async fn initialise() -> JoinHandle<()> {
        let my_local_ip = local_ip().unwrap();
        is_anyone_home(my_local_ip, 100, 6969).await;

        println!("Client activity initiating...");
        let client_thread: JoinHandle<()> = tokio::spawn(async {
            loop {
                client_iteration().await;
            }
        });
        println!("Client activity initiated...");
        client_thread
    }
}

pub async fn client_iteration() {
    thread::sleep(Duration::from_secs(1));

    println!("Number of addresses {}", GLOBAL_DATA.lock().unwrap().len());
    let temp_addresses = GLOBAL_DATA.lock().unwrap().clone();

    for address in temp_addresses {
        println!("Acking {}", address);
        dispatch_ack(address).await;
    }
}

pub async fn is_anyone_home(own_ip: IpAddr, max_range: u8, port: u32) {
    let ip_ending = own_ip
        .to_string()
        .split('.')
        .last()
        .map(String::from)
        .unwrap()
        .parse::<u8>()
        .unwrap();

    let ip_range = 1..max_range;
    let mut ips: Vec<u8> = ip_range.collect();
    ips.remove((ip_ending - 1) as usize);

    let mut futs = vec![];

    for ip in ips {
        futs.push(async move {
            let address = Ipv4Addr::new(192, 168, 0, ip);
            let formatted_address = format!("http://{}:{}/ping", address, port);
            dispatch_ping(formatted_address).await
        });
    }
    futures::future::join_all(futs).await;
}

async fn dispatch_ack(address: String) {
    let url = format!("http://{}:6969/ack", address);
    let response = CLIENT
        .get(url)
        .header("Content-Type", "text/plain")
        .send()
        .await;

    match response {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error is {}", e);
            remove_from_list(address)
        }
    }
}

async fn dispatch_ping(address: String) {
    let response = CLIENT
        .get(address)
        .timeout(Duration::from_secs(2))
        .header("Content-Type", "text/plain")
        .send()
        .await;

    handle_ping_response(response).await;
}

async fn handle_ping_response(response: Result<reqwest::Response, reqwest::Error>) {
    let mut addresses: HashSet<String> = HashSet::new();
    if let Ok(result) = response {
        match result.text().await {
            Ok(body_text) => {
                let dad: Vec<String> = body_text.split(',').map(String::from).collect();
                addresses.extend(dad);
            }
            Err(err) => {
                eprint!("Couldn't process csv data of addresses");
                eprint!("{}", err);
            }
        };
    };

    initialize_list(addresses);
}

fn initialize_list(addresses: HashSet<String>) {
    GLOBAL_DATA.lock().unwrap().extend(addresses);
}

fn remove_from_list(address: String) {
    println!("Removing {} from list", address);
    GLOBAL_DATA.lock().unwrap().remove(address.as_str());
}
