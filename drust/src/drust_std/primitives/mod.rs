use crate::conf::*;
use crate::drust_std::comm::*;
use std::mem;

pub mod dbox;
pub mod dmut;
pub mod dref;
pub mod tbox;

#[derive(PartialEq, Eq)]
pub enum Destination {
    Local,
    Remote(usize),
}

pub trait DRust {
    fn static_typeid() -> u32 where Self: Sized;
    fn typeid(&self) -> u32;
    fn migrate(&mut self, _dst: Destination) -> bool;
}

// It means it has no local embedded data
#[macro_export]
macro_rules! exclude {
    ($($t:ty, $tid:expr),*) => {
        $(impl DRust for $t {
            fn static_typeid() -> u32 {
                $tid
            }
            fn typeid(&self) -> u32 {
                $tid
            }
            fn migrate(&mut self, _dst: Destination) -> bool {false}
        })*
    }
}

pub fn consume_original_data<T>(
    data: &mut Option<Box<T, &'static good_memory_allocator::SpinLockedAllocator>>,
) -> *mut T {
    let original_data = mem::replace(data, None);
    let raw_1 = Box::into_raw(original_data.unwrap());
    raw_1
}

pub fn current_place(addr: usize) -> Destination {
    if addr >= GLOBAL_HEAP_START && addr < GLOBAL_HEAP_START + WORKER_HEAP_SIZE {
        let server_idx = (addr - GLOBAL_HEAP_START) / WORKER_UNIT_SIZE;
        if server_idx == unsafe { SERVER_INDEX } {
            Destination::Local
        } else {
            Destination::Remote(server_idx)
        }
    } else {
        panic!("Address {:x} is not in any heap", addr);
    }
}

// If the region_size is larger than 1GB, then split it into multiple 1GB regions and read them to local
pub fn drust_read_large_sync(
    local_dst_offset: usize,
    remote_src_offset: usize,
    region_size: usize,
    tid: usize,
) {
    let mut region_size = region_size;
    let mut local_dst_offset = local_dst_offset;
    let mut remote_src_offset = remote_src_offset;
    while region_size > 0 {
        let size = if region_size > 1 << 30 {
            1 << 30
        } else {
            region_size
        };
        let tid = std::thread::current().id().as_u64().get() as usize;
        unsafe { drust_read_sync(local_dst_offset, remote_src_offset, size, tid) };
        region_size -= size;
        local_dst_offset += size;
        remote_src_offset += size;
    }
}

// If the region_size is larger than 1GB, then split it into multiple 1GB regions and write them to remote
pub fn drust_write_large_sync(
    local_src_offset: usize,
    remote_dst_offset: usize,
    region_size: usize,
    tid: usize,
) {
    let mut region_size = region_size;
    let mut local_src_offset = local_src_offset;
    let mut remote_dst_offset = remote_dst_offset;
    while region_size > 0 {
        let size = if region_size > 1 << 30 {
            1 << 30
        } else {
            region_size
        };
        unsafe { drust_write_sync(local_src_offset, remote_dst_offset, size, tid) };
        region_size -= size;
        local_src_offset += size;
        remote_dst_offset += size;
    }
}
