mod client;
mod server;

use crate::client::LilNetClient;
use crate::server::LilNetServer;

use std::{collections::BTreeSet, sync::Mutex};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GLOBAL_DATA: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
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
    let server = LilNetServer::initialise().await;
    let client = LilNetClient::initialise().await;

    loop {
        if server.is_finished() && client.is_finished() {
            break;
        }
    }

    Ok(())
}
