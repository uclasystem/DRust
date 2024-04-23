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

pub mod app;
pub mod conf;
pub mod drust_std;
pub mod prelude;
pub mod spec;

use conf::APPLICATION_NAME;
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let addrs = drust_std::prepare();
    drust_std::run(addrs);
    drust_std::finalize();
}
