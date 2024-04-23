use dashmap::DashMap;

pub const NUM_SERVERS: usize = 1;
pub const SERVER_INDEX: usize = 0;
pub const READ_RATIO: i32 = 50;
pub const TARGET_BUCKET_NUM : usize = 16777216;
pub const TAG_BITS: usize = 11;
pub const BKT_BITS: usize = 24;
pub const BKT_MASK: usize = (1 << BKT_BITS) - 1;
pub const UNIT_BUCKET_NUM: usize = (16777216 - 1)/ NUM_SERVERS + 1;
pub const BUCKET_NUM: usize = UNIT_BUCKET_NUM * NUM_SERVERS;

pub const THREAD_NUM: usize = 1;
pub const UNIT_THREAD_BUCKET_NUM: usize = (UNIT_BUCKET_NUM - 1) / THREAD_NUM + 1;

pub static mut LOCAL_CACHE: Option<DashMap<usize, usize>> = None;
pub static mut GLOBAL_MAP_ADDR: [usize; NUM_SERVERS] = [0; NUM_SERVERS];


pub fn bucket(key: usize) -> usize {
    let new_key = (key >> TAG_BITS);
    new_key & BKT_MASK
}