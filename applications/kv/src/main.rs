#![feature(mutex_unlock)]
use tokio::runtime::Runtime;

pub mod entry;
pub mod conf;
pub mod benchmark;
pub mod dmap;


// load column from file and return a Column struct
pub async fn run() {
    benchmark::zipf_bench().await;
}

fn main() {
    Runtime::new().unwrap().block_on(run());
}