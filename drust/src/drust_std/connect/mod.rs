pub mod dsafepoint;

use crate::drust_std::alloc::init::{init_connections, init_heap};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::{net::SocketAddr, sync::Arc, thread::sleep};
use tarpc::tokio_serde::formats::Json;
// use tarpc::server;
use crate::drust_std::comm::*;
use dsafepoint::*;
use tokio::runtime::Runtime;

use crate::conf::*;

#[derive(Serialize, Deserialize)]
struct Server {
    ip: String,
    alloc_ip: String,
    safepoint_ip: String,
}
#[derive(Serialize, Deserialize)]
struct Config {
    servers: Vec<Server>,
}

#[macro_export]
macro_rules! drun_server {
    ($addr:expr, $appserver:expr) => {
        let mut listener = tarpc::serde_transport::tcp::listen(&($addr), Json::default)
            .await
            .unwrap();
        listener.config_mut().max_frame_length(usize::MAX);
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            // Limit channels to 1 per IP.
            .max_channels_per_key(1000, |t| t.transport().peer_addr().unwrap().ip())
            // serve is generated by the service attribute. It takes as input any type implementing
            // the generated World trait.
            .map(|channel| {
                let server = $appserver(channel.transport().peer_addr().unwrap());
                channel.execute(server.serve())
            })
            // Max 10 channels.
            .buffer_unordered(10000)
            .for_each(|_| async {})
            .await;
    };
}

#[macro_export]
macro_rules! dconnect {
    ($addr:expr, $vec:ident, $appClient:ident) => {
        let mut client_refs: Vec<Arc<$appClient>> = Vec::with_capacity(NUM_SERVERS);
        for i in 0..NUM_SERVERS {
            let mut transport = tarpc::serde_transport::tcp::connect(&$addr[i], Json::default);
            transport.config_mut().max_frame_length(usize::MAX);
            let fut_transport = transport.await.expect("failed to connect");
            let config = Config::default();
            client_refs.push(Arc::new($appClient::new(config, fut_transport).spawn()));
            println!("connected to server {}", i);
        }
        unsafe {
            $vec = Some(client_refs);
        }
    };
}

pub fn get_server_addrs() -> (
    [SocketAddr; NUM_SERVERS],
    [SocketAddr; NUM_SERVERS],
    [SocketAddr; NUM_SERVERS],
) {
    let config =
        serde_json::from_str::<Config>(&std::fs::read_to_string("drust.json").unwrap()).unwrap();
    let mut server_addrs = [SocketAddr::from(([0, 0, 0, 0], 0)); NUM_SERVERS];
    let mut alloc_server_addrs = [SocketAddr::from(([0, 0, 0, 0], 0)); NUM_SERVERS];
    let mut safepoint_addrs = [SocketAddr::from(([0, 0, 0, 0], 0)); NUM_SERVERS];
    for i in 0..NUM_SERVERS {
        server_addrs[i] = config.servers[i].ip.parse().unwrap();
        alloc_server_addrs[i] = config.servers[i].alloc_ip.parse().unwrap();
        safepoint_addrs[i] = config.servers[i].safepoint_ip.parse().unwrap();
    }
    (server_addrs, alloc_server_addrs, safepoint_addrs)
}

pub fn rconnect(alloc_addr: SocketAddr) {
    println!("[rust] start");
    let server_idx = unsafe { SERVER_INDEX };

    if NUM_SERVERS > 1 {
        let s_idx = server_idx;
        std::thread::spawn(move || {
            println!("start drust rdma server from the spawned thread!");
            unsafe {
                drust_start_server(LOCAL_HEAP_START, WORKER_UNIT_SIZE, s_idx);
            }
        });
        // TODO: wait until the server is ready
        unsafe {
            drust_server_ready();
        }
    } else {
        unsafe {
            register_mem(LOCAL_HEAP_START, WORKER_HEAP_SIZE);
        }
    }

    let s_addr = alloc_addr;
    std::thread::spawn(move || {
        println!("start drust distributed alloc server from the spawned thread!");
        init_heap(s_addr);
    });
    sleep(std::time::Duration::from_secs(1));
    set_ready(0);
}

pub fn rconnect_alloc(alloc_addrs: [SocketAddr; NUM_SERVERS]) {
    std::thread::spawn(move || {
        println!("start connecting distributed alloc from the spawned thread!");
        Runtime::new()
            .unwrap()
            .block_on(init_connections(alloc_addrs));
    });
}
