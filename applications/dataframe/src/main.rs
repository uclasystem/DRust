#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(vec_into_raw_parts)]
#![feature(thread_id_value)]
#![feature(concat_idents)]
#![feature(async_fn_in_trait)]
#![allow(unused_imports)]
#![allow(unused)]
#![allow(dead_code)]
#![feature(unsafe_pin_internals)]
#![feature(core_intrinsics)]
#![feature(ptr_from_ref)]

pub mod prelude;
pub mod benchmark;
pub mod utils;
pub mod chunked_array;
pub mod error;
pub mod self_arrow;
pub mod datatypes;
pub mod series;
pub mod frame;


use std::time::Instant;
use tokio::runtime::Runtime;

use benchmark::{utils::*, groupby::h2oai_groupby_benchmark};
use utils::*;




pub const DATASET_ID: usize = 2;
pub const DATASET_NAME: &str = "G1_1e8_1e2_0_0.csv";

pub async fn run() {
    unsafe{
        COMPUTES = Some(ResourceManager::new(1));
    }
    match DATASET_ID {
        0 => h2oai_groupby_benchmark(DSize::Small).await,
        1 => h2oai_groupby_benchmark(DSize::Medium).await,
        2 => h2oai_groupby_benchmark(DSize::Large).await,
        3 => h2oai_groupby_benchmark(DSize::Huge).await,
        _ => panic!("wrong dataset id"),
    }

}

fn main() {
    Runtime::new().unwrap().block_on(run());
}
