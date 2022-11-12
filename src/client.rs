use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use crate::{CLIENT, GLOBAL_DATA};

pub async fn client_iteration() {
    println!("Doing client shit!");
}

pub async fn is_anyone_home(own_ip: IpAddr, max_range: u8, port: u32) {
    let ip_range = 1..max_range;

    // TODO Remove own IP from assessment
    let ips: Vec<u8> = ip_range.collect();

    let mut futs = vec![];

    for ip in ips {
        futs.push(async move {
            let address = Ipv4Addr::new(192, 168, 0, ip);
            let formatted_address = format!("http://{}:{}", address.to_string(), port.to_string());
            let request_body = format!("{own_ip}");

            hello_request(formatted_address, request_body).await
        });
    }

    futures::future::join_all(futs).await;
}

async fn hello_request(address: String, request_body: String) {
    let response = CLIENT
        .post(&address)
        .timeout(Duration::from_secs(2))
        .body(request_body)
        .header("Content-Type", "text/plain")
        .send()
        .await;

    let addresses = handle_response(response).await;

    initialize_list(addresses);
}

async fn handle_response(response: Result<reqwest::Response, reqwest::Error>) -> HashSet<String> {
    return match response {
        Ok(result) => {
            let body = result.text().await;
            let addresses: HashSet<String> = match body {
                Ok(body_text) => {
                    let mut reader = csv::Reader::from_reader(body_text.as_bytes());

                    let mut addresses: HashSet<String> = HashSet::new();

                    for record in reader.records() {
                        match record {
                            Ok(val) => {
                                addresses.extend(val.into_iter().map(|data| String::from(data)))
                            }
                            Err(err) => {
                                eprint!("Couldn't process csv data of addresses");
                                eprint!("{}", err);
                            }
                        };
                    }
                    addresses
                }
                Err(err) => {
                    eprint!("Couldn't process csv data of addresses");
                    eprint!("{}", err);
                    HashSet::new()
                }
            };

            addresses
        }

        Err(_) => HashSet::new(),
    };
}

fn initialize_list(addresses: HashSet<String>) {
    GLOBAL_DATA.lock().unwrap().extend(addresses);
}
