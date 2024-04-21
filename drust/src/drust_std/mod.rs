pub mod alloc;
pub mod collections;
pub mod comm;
pub mod connect;
pub mod primitives;
pub mod sync;
pub mod thread;
pub mod utils;

use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};

use crate::{
    app,
    conf::*,
    dconnect,
    drust_std::{
        connect::dsafepoint::{
            rshutdown, set_ready, start_safepoint_server, READY_MAP, SHUTDOWN_INDEX,
        },
        thread::dlink::{run_server, DRustWorldClient, DCLIENTS},
    },
};
use clap::Parser;
use connect::dsafepoint::rsync;
use tarpc::{
    client::{self, Config},
    context, server,
    server::{incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};
use tokio::runtime::Runtime;
use tokio::time::sleep;
use utils::*;

pub fn prepare() -> (
    [SocketAddr; NUM_SERVERS],
    [SocketAddr; NUM_SERVERS],
    [SocketAddr; NUM_SERVERS],
) {
    let (app, server_idx) = get_args();
    unsafe {
        SERVER_INDEX = server_idx;
        LOCAL_HEAP_START = GLOBAL_HEAP_START + server_idx * WORKER_UNIT_SIZE;
        APPLICATION_NAME = Some(app);
    }
    let (app_addrs, alloc_addrs, safepoint_addrs) = connect::get_server_addrs();

    let mut sync_server_addr = safepoint_addrs[server_idx];
    std::thread::spawn(move || {
        println!("start sync server from the spawned thread!");
        Runtime::new()
            .unwrap()
            .block_on(start_safepoint_server(sync_server_addr))
            .unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));

    connect::rconnect(alloc_addrs[server_idx]);
    Runtime::new().unwrap().block_on(rsync(&safepoint_addrs, 0));
    (app_addrs, alloc_addrs, safepoint_addrs)
}

pub async fn drust_main(
    app_addrs: [SocketAddr; NUM_SERVERS],
    safepoint_addrs: [SocketAddr; NUM_SERVERS],
    server_idx: usize,
) {
    let app = unsafe { APPLICATION_NAME.as_ref().unwrap().clone() };
    std::thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(run_server(app_addrs[server_idx]));
    });
    sleep(Duration::from_secs(2)).await;
    set_ready(2);
    rsync(&safepoint_addrs, 1).await;
    rsync(&safepoint_addrs, 2).await;
    dconnect!(app_addrs, DCLIENTS, DRustWorldClient);
    set_ready(3);
    rsync(&safepoint_addrs, 3).await;
    if server_idx == 0 {
        if app == "gemm" {
            app::gemm::run().await;
        } else if app == "dataframe" {
            app::dataframe::run().await;
        } else if app == "kv" {
            app::kv::run().await;
        } else if app == "sn" {
            app::socialnet::run().await;
        } else {
            panic!("unknown app");
        }
        
        println!("drust_main done");
        rshutdown(&safepoint_addrs).await;
    } else {
        loop {
            sleep(Duration::from_secs(5)).await;
            let ready_map = unsafe { READY_MAP.as_ref().unwrap() };
            if ready_map.get(&SHUTDOWN_INDEX).is_some() {
                break;
            }
        }
    }
}

pub fn run(
    addrs: (
        [SocketAddr; NUM_SERVERS],
        [SocketAddr; NUM_SERVERS],
        [SocketAddr; NUM_SERVERS],
    ),
) {
    let (app_addrs, alloc_addrs, safepoint_addrs) = addrs;
    let app = unsafe { APPLICATION_NAME.as_ref().unwrap().clone() };
    let safepoint_addrs_copy = safepoint_addrs.clone();
    let server_idx = unsafe { SERVER_INDEX };
    println!("drust {} started", app);
    let handle = std::thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(drust_main(app_addrs, safepoint_addrs_copy, server_idx));
    });
    connect::rconnect_alloc(alloc_addrs);
    std::thread::sleep(Duration::from_secs(2));
    handle.join().unwrap();
}

pub fn finalize() {
    println!("finalize");
}
