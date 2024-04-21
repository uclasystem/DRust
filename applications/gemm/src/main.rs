#![feature(allocator_api)]
pub mod conf;
pub mod matrix;
pub mod par_strassen;
pub mod single_strassen;
pub mod utils;

use std::time::Instant;
use tokio::runtime::Runtime;

use conf::*;
use par_strassen::*;
use utils::*;
pub async fn run() {
    unsafe {
        BRANCHES = Some(ResourceManager::new(BRANCH_NUM));
        COMPUTES = Some(ResourceManager::new(THREADS_NUM));
    }
    let mut matrix_a = Vec::with_capacity(MATRIX_SIZE * MATRIX_SIZE);
    let mut matrix_b = Vec::with_capacity(MATRIX_SIZE * MATRIX_SIZE);
    for _ in 0..MATRIX_SIZE {
        for _ in 0..MATRIX_SIZE {
            matrix_a.push(1 as i32);
            matrix_b.push(1 as i32);
        }
    }

    let start_time = Instant::now();

    let matrix_c = par_strassen_mul(matrix_a, matrix_b, MATRIX_SIZE, 1).await;

    let duration = start_time.elapsed();
    println!(
        "Time elapsed in matrix multiplication function() is: {:?}",
        duration
    );
    for i in (MATRIX_SIZE - 16)..MATRIX_SIZE {
        for j in (MATRIX_SIZE - 16)..MATRIX_SIZE {
            println!("matrix_c[{}, {}] = {}", i, j, matrix_c[i * 16 + j]);
        }
    }
}

fn main() {
    Runtime::new().unwrap().block_on(run());
}
