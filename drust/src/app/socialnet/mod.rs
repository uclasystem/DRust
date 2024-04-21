pub mod text;
pub mod storage;
pub mod post;
pub mod compose;
pub mod conf;
pub mod benchmark;
pub mod media;
pub mod uniqueid;

use std::{net::SocketAddr, io, time::{SystemTime, Duration, UNIX_EPOCH}, sync::{Arc, atomic::AtomicU64}};
use dashmap::DashMap;
use tokio::runtime::Runtime;

use conf::*;
use uniqueid::UNIQUE_ID_CACHE;
use benchmark::socialnet_benchmark;

use crate::{conf::SERVER_INDEX, prelude::*};





pub async fn run() {
    if unsafe{SERVER_INDEX} == 0 {
        socialnet_benchmark().await;
    }
}
