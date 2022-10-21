use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{parse_response, GLOBAL_DATA};

pub async fn process_new_joiner(mut socket: TcpStream) {
    let request = parse_response(socket).await;

    let request_body = request.body;

    match request_body {
        Some(address) => {
            println!("Request body is: {}", address);
            add_to_own_list(address);
        }
        None => println!("No request body"),
    }

    if let Err(e) = socket.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await {
        eprintln!("failed to write to socket; err = {:?}", e);
        return;
    }

    ()
}

fn add_to_own_list(address: String) {
    GLOBAL_DATA.lock().unwrap().insert(address);
}
