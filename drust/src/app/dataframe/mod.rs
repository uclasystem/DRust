pub mod prelude;
pub mod benchmark;
pub mod utils;
pub mod chunked_array;
pub mod error;
pub mod self_arrow;
pub mod datatypes;
pub mod series;
pub mod frame;



use benchmark::{utils::*, groupby::h2oai_groupby_benchmark};
use crate::drust_std::utils::*;
use crate::conf::*;




pub const DATASET_ID: usize = 2;
pub const DATASET_NAME: &str = "G1_1e8_1e2_0_0.csv";

pub async fn run() {
    unsafe{
        COMPUTES = Some(ResourceManager::new(NUM_SERVERS));
    }
    match DATASET_ID {
        0 => h2oai_groupby_benchmark(DSize::Small).await,
        1 => h2oai_groupby_benchmark(DSize::Medium).await,
        2 => h2oai_groupby_benchmark(DSize::Large).await,
        3 => h2oai_groupby_benchmark(DSize::Huge).await,
        _ => panic!("wrong dataset id"),
    }

}