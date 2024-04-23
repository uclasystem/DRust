pub mod dlink;

use std::{
    mem::transmute,
    ptr::{self, copy_nonoverlapping}, time::{Duration, SystemTime},
};

use crate::drust_std::{NUM_SERVERS, RPC_WAIT};
use futures::{Future, FutureExt};
use tarpc::context;
use tokio::task::JoinHandle;

use self::dlink::get_dclient;

use super::{collections::dvec::DVec, primitives::DRust, Resource, SimpleResource, COMPUTES, GLOBAL_HEAP_START, SIMPLE_COMPUTES, WORKER_UNIT_SIZE};

pub fn inner_spawn<F, T>(future: F, resource: Resource) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    let mut f = future.boxed();
    let siz = std::mem::size_of_val(&(*f));
    let mut values = unsafe { transmute::<_, (usize, usize)>(f.pointer) };

    let mut stack_vec = vec![0u8; siz];
    unsafe {
        let (addr, mut len, cap) = stack_vec.into_raw_parts();
        let orig_addr = values.0 as *mut u8;
        copy_nonoverlapping(orig_addr, addr, siz);
        len = siz;
        stack_vec = Vec::from_raw_parts(addr, len, cap);
    }
    tokio::spawn(async move {
        let client = get_dclient(resource.id % NUM_SERVERS);
        let mut ctx = context::current();
        ctx.deadline = SystemTime::now() + Duration::from_secs(RPC_WAIT);
        let p = client.remote_spawn(ctx, values, stack_vec, T::static_typeid());
        let return_vec = p.await.unwrap();
        let mut return_vals = T::default();
        let siz = std::mem::size_of_val(&return_vals);
        assert!(siz == return_vec.len());
        unsafe {
            let addr = &mut return_vals as *mut T as *mut u8;
            copy_nonoverlapping(return_vec.as_ptr(), addr, siz);
        }
        resource.release();
        return_vals
    })

}

pub fn inner_spawn_strict<F, T>(future: F, resource: Option<SimpleResource>, server_idx: usize) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    let mut f = future.boxed();
    let siz = std::mem::size_of_val(&(*f));
    let mut values = unsafe { transmute::<_, (usize, usize)>(f.pointer) };
    let mut stack_vec = vec![0u8; siz];
    unsafe {
        let (addr, mut len, cap) = stack_vec.into_raw_parts();
        let orig_addr = values.0 as *mut u8;
        copy_nonoverlapping(orig_addr, addr, siz);
        len = siz;
        stack_vec = Vec::from_raw_parts(addr, len, cap);
    }
    tokio::spawn(async move {
        let client = get_dclient(server_idx % NUM_SERVERS);
        let mut ctx = context::current();
        ctx.deadline = SystemTime::now() + Duration::from_secs(RPC_WAIT);
        let p = client.remote_spawn(ctx, values, stack_vec, T::static_typeid());
        let return_vec = p.await.unwrap();
        let mut return_vals = T::default();
        let siz = std::mem::size_of_val(&return_vals);
        assert!(siz == return_vec.len());
        unsafe {
            let addr = &mut return_vals as *mut T as *mut u8;
            copy_nonoverlapping(return_vec.as_ptr(), addr, siz);
        }
        match resource {
            Some(resource) => resource.release(),
            None => {}
        }
        return_vals
    })
}


pub fn dspawn<F, T>(future: F) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    
    let thread_manager = unsafe { COMPUTES.as_ref().unwrap() };
    let resource = thread_manager.get_resource(0);
    inner_spawn(future, resource)
}

pub fn dspawn_to<F, T>(future: F, addr: usize) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    let server_idx = (addr - GLOBAL_HEAP_START) / WORKER_UNIT_SIZE;
    let thread_manager = unsafe { COMPUTES.as_ref().unwrap() };
    let resource = thread_manager.get_resource(server_idx);
    inner_spawn(future, resource)
}


pub fn dspawn_to_strictly<F, T>(future: F, server_idx: usize) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    match unsafe { SIMPLE_COMPUTES.as_ref() } {
        None => {
            inner_spawn_strict(future, None, server_idx)
        },
        Some(thread_managers) => {
            let thread_manager = thread_managers.get(server_idx).unwrap();
            let resource = thread_manager.get_resource();
            inner_spawn_strict(future, Some(resource), server_idx)
        }
    }
}

pub fn dspawn_to_relaxed<F, T>(future: F, server_idx: usize) -> JoinHandle<T>
where
    T: DRust + Send + Default + 'static,
    F: Future + Send,
    F::Output: Send,
{
    inner_spawn_strict(future, None, server_idx)
}



pub async fn dscope_spawn<F, T>(future: F) -> T
where
    T: DRust + Send + Default,
    F: Future + Send,
    F::Output: Send,
{
    
    let thread_manager = unsafe { COMPUTES.as_ref().unwrap() };
    let resource = thread_manager.get_resource(0);

    let mut f = future.boxed();
    let siz = std::mem::size_of_val(&(*f));
    let mut values = unsafe { transmute::<_, (usize, usize)>(f.pointer) };

    let mut stack_vec = vec![0u8; siz];
    unsafe {
        let (addr, mut len, cap) = stack_vec.into_raw_parts();
        let orig_addr = values.0 as *mut u8;
        copy_nonoverlapping(orig_addr, addr, siz);
        len = siz;
        stack_vec = Vec::from_raw_parts(addr, len, cap);
    }
    let client = get_dclient(resource.id % NUM_SERVERS);
    let mut ctx = context::current();
    ctx.deadline = SystemTime::now() + Duration::from_secs(RPC_WAIT);
    let p = client.remote_spawn(ctx, values, stack_vec, T::static_typeid());
    let return_vec = p.await.unwrap();
    let mut return_vals = T::default();
    let siz = std::mem::size_of_val(&return_vals);
    assert!(siz == return_vec.len());
    unsafe {
        let addr = &mut return_vals as *mut T as *mut u8;
        copy_nonoverlapping(return_vec.as_ptr(), addr, siz);
    }
    resource.release();
    return_vals
}
