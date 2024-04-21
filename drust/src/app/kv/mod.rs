use crate::{conf::NUM_SERVERS, drust_std::utils::{ResourceManager, COMPUTES}};

pub mod entry;
pub mod conf;
pub mod benchmark;
pub mod dmap;


// load column from file and return a Column struct
pub async fn run() {
    unsafe{
        COMPUTES = Some(ResourceManager::new(NUM_SERVERS));
    }
    benchmark::zipf_bench().await;
}
