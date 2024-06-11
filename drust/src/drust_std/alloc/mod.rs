use std::{
    alloc::{Allocator, Layout},
    net::SocketAddr,
    ptr::{self, NonNull},
    sync::{Arc, Once},
};

use dashmap::DashMap;
use tarpc::context;
use tokio::runtime::Runtime;

use crate::{
    conf::*,
    dprintln,
    drust_std::alloc::init::start_alloc_server,
    spec::{drop_vec_with_id, from_id_to_type},
};

pub mod init;

pub static mut LOCAL_ALLOCATOR: good_memory_allocator::SpinLockedAllocator =
    good_memory_allocator::SpinLockedAllocator::empty();
static mut DALLOCTOR: Option<Vec<Arc<DAllocatorClient>>> = None;
pub static mut REMOTE_ALLOCATORS: Option<Vec<usize>> = None;
static INIT: Once = Once::new();

pub static mut REF_MAP: Option<Arc<DashMap<usize, (usize, usize)>>> = None;

// pub static mut RDROP_CNT: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

#[tarpc::service]
pub trait DAllocator {
    /// Returns a greeting for name.
    async fn rallocate(size: usize, alignment: usize) -> (usize, usize);
    async fn rdeallocate(addr: usize, size: usize, alignment: usize);
    async fn rdrop(addr: usize, type_id: usize);
    async fn get_allocator() -> usize;
    async fn rdrop_vec(addr: usize, capacity: usize, len: usize, type_id: usize);
    async fn rupdate(owner_addr: usize, data_addr: usize);
}

#[derive(Clone)]
struct DAllocServer(SocketAddr);

#[tarpc::server]
impl DAllocator for DAllocServer {
    async fn rallocate(self, _: context::Context, size: usize, alignment: usize) -> (usize, usize) {
        // println!("allocate size: {}, alignment: {}", size, alignment);
        let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
        let alloc_ptr = unsafe { LOCAL_ALLOCATOR.allocate(layout) };
        match alloc_ptr {
            Ok(ptr) => {
                let size = ptr.len();
                let addr = ptr.as_mut_ptr() as usize;
                (addr, size)
            }
            Err(_e) => (0, 0),
        }
    }
    async fn rdeallocate(self, _: context::Context, addr: usize, size: usize, alignment: usize) {
        // unsafe {
        //     let mut cnt = RDROP_CNT.lock().unwrap();
        //     *cnt += 1;
        //     if *cnt % 100 == 0 {
        //         println!("rdrop cnt: {}", *cnt);
        //     }
        // }
        dprintln!(
            "deallocate addr: {}, size: {}, alignment: {}",
            addr,
            size,
            alignment
        );
        let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
        unsafe { LOCAL_ALLOCATOR.deallocate(NonNull::new_unchecked(addr as *mut u8), layout) };
    }
    async fn rdrop(self, _: context::Context, addr: usize, type_id: usize) {
        dprintln!("drop addr: {}, type_id: {}", addr, type_id);
        let obj = from_id_to_type(type_id as u32, addr);
        drop(obj);
    }
    async fn get_allocator(self, _: context::Context) -> usize {
        unsafe { &LOCAL_ALLOCATOR as *const _ as usize }
    }
    async fn rdrop_vec(
        self,
        _: context::Context,
        addr: usize,
        capacity: usize,
        len: usize,
        type_id: usize,
    ) {
        dprintln!("drop vec addr: {}, type_id: {}", addr, type_id);
        drop_vec_with_id(type_id as u32, addr, capacity, len);
    }

    async fn rupdate(self, _: context::Context, owner_addr: usize, data_addr: usize) {
        unsafe {
            let new_box = Some(Box::from_raw_in(data_addr as *mut u8, &LOCAL_ALLOCATOR,));
            ptr::write_volatile(owner_addr as *mut Option<Box<u8, &good_memory_allocator::SpinLockedAllocator<20, 8>>>, new_box);
        }
    }
}

pub unsafe fn get_remote_allocator(
    index: usize,
) -> &'static good_memory_allocator::SpinLockedAllocator {
    &*(REMOTE_ALLOCATORS.as_ref().unwrap()[index]
        as *const good_memory_allocator::SpinLockedAllocator)
}

#[derive(Debug, Clone)]
pub struct AllocError;

pub fn dallocate(layout: Layout, server_idx: usize) -> Result<NonNull<[u8]>, AllocError> {
    let size = layout.size();
    let alignment = layout.align();
    let client = Arc::clone(&unsafe { DALLOCTOR.as_ref().unwrap() }[server_idx]);
    let (addr, allocated_size) = std::thread::spawn(move || {
        Runtime::new()
            .unwrap()
            .block_on(client.rallocate(context::current(), size, alignment))
    })
    .join()
    .unwrap()
    .unwrap();
    if allocated_size < size {
        return Err(AllocError);
    } else {
        unsafe {
            Ok(NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(addr as *mut u8),
                layout.size(),
            ))
        }
    }
}

pub fn ddeallocate(ptr: NonNull<u8>, layout: Layout, server_idx: usize) {
    let size = layout.size();
    let alignment = layout.align();
    let client = Arc::clone(&unsafe { DALLOCTOR.as_ref().unwrap() }[server_idx]);
    // Runtime::new().unwrap().block_on(client.rdeallocate(context::current(), ptr.as_ptr() as usize, size, alignment)).unwrap();
    let ptr_raw = ptr.as_ptr() as usize;
    tokio::spawn(async move {
        client
            .rdeallocate(context::current(), ptr_raw, size, alignment)
            .await
            .unwrap();
    });
    // std::thread::spawn(move || {
    //     Runtime::new().unwrap().block_on(client.rdeallocate(context::current(), ptr_raw, size, alignment))
    // });
}

pub fn ddrop(ptr: NonNull<u8>, type_id: usize, server_idx: usize) {
    dprintln!("Dropping remote by dispatch remote drop call");
    let client = Arc::clone(&unsafe { DALLOCTOR.as_ref().unwrap() }[server_idx]);
    let ptr_raw = ptr.as_ptr() as usize;
    tokio::spawn(async move {
        client
            .rdrop(context::current(), ptr_raw, type_id)
            .await
            .unwrap();
    });
    // std::thread::spawn(move || {
    //     Runtime::new().unwrap().block_on(client.rdrop(context::current(), ptr_raw, type_id))
    // });
}

pub fn ddrop_vec(ptr: NonNull<u8>, type_id: usize, capacity: usize, len: usize, server_idx: usize) {
    let client = Arc::clone(&unsafe { DALLOCTOR.as_ref().unwrap() }[server_idx]);
    // Runtime::new().unwrap().block_on(client.rdrop_vec(context::current(), ptr.as_ptr() as usize, capacity, len, type_id)).unwrap();
    let ptr_raw = ptr.as_ptr() as usize;
    tokio::spawn(async move {
        client
            .rdrop_vec(context::current(), ptr_raw, capacity, len, type_id)
            .await
            .unwrap();
    });
    // std::thread::spawn(move || {
    //     Runtime::new().unwrap().block_on(client.rdrop_vec(context::current(), ptr_raw, capacity, len, type_id))
    // });
}

pub fn dupdate(owner_addr: usize, data_addr: usize, server_idx: usize) {
    let client = Arc::clone(&unsafe { DALLOCTOR.as_ref().unwrap() }[server_idx]);
    let owner_addr = owner_addr as usize;
    let data_addr = data_addr as usize;
    tokio::spawn(async move {
        client
            .rupdate(context::current(), owner_addr, data_addr)
            .await
            .unwrap();
    });
}

pub fn init_heap(server_addr: SocketAddr) {
    let heap_start = GLOBAL_HEAP_START + unsafe { SERVER_INDEX } * WORKER_UNIT_SIZE;
    let buffer_size = WORKER_UNIT_SIZE;
    dprintln!("heap_start: {:x}", heap_start);
    unsafe {
        LOCAL_ALLOCATOR.init(heap_start, buffer_size);
    }
    dprintln!("local allocator: {:x}", unsafe {
        &LOCAL_ALLOCATOR as *const _ as usize
    });
    Runtime::new()
        .unwrap()
        .block_on(start_alloc_server(server_addr))
        .unwrap();
}
