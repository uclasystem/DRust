use std::{
    mem::transmute,
    net::SocketAddr,
    pin::Pin,
    ptr::{self, copy_nonoverlapping, from_mut},
    sync::Arc,
};

use futures::{future, prelude::*};
use tarpc::{
    client::{self, Config},
    context, server,
    server::{incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};
use tokio::runtime::Runtime;

use crate::{
    app::{dataframe::prelude::Chunk, gemm::par_strassen::{par_strassen_mul, single_strassen_mul}, socialnet::media::Image}, dprintln, drun_server, drust_std::{alloc::LOCAL_ALLOCATOR, collections::dvec::*}
};

#[tarpc::service]
pub trait DRustWorld {
    async fn remote_spawn(
        ptr: (usize, usize),
        future_bytes: Vec<u8>,
        typeid: u32,
    ) -> Vec<u8>;
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct DRustServer(SocketAddr);

#[tarpc::server]
impl DRustWorld for DRustServer {
    async fn remote_spawn(
        self,
        _: context::Context,
        ptr: (usize, usize),
        future_bytes: Vec<u8>,
        typeid: u32,
    ) -> Vec<u8> {
        let siz = future_bytes.len();
        let mut new_ptr = ptr;
        unsafe {
            let new_addr = vec![0u8; siz];
            let (new_addr, new_len, new_cap) = new_addr.into_raw_parts();
            unsafe { copy_nonoverlapping(future_bytes.as_ptr(), new_addr, siz) };
            new_ptr.0 = new_addr as usize;
            // println!("new_ptr: ({}, {}), siz: {}", new_ptr.0, new_ptr.1, siz);
        }
        if (typeid & 0xFF) == 1 {
            let sub_typeid = typeid >> 8;
            if (sub_typeid & 0xFF) == 1 {
                let subsub_typeid = sub_typeid >> 8;
                if subsub_typeid == 19 {
                    let p = unsafe {
                        transmute::<(usize, usize), Box<dyn Future<Output = DVec<DVec<u8>>> + Send>>(new_ptr)
                    };
                    let f = Pin::from(p);
                    let mut v = f.await;
                    let siz = std::mem::size_of_val(&v);
                    let mut return_vals = Vec::with_capacity(siz);
                    unsafe {
                        let (addr, len, cap) = return_vals.into_raw_parts();
                        let orig_addr = &v as *const DVec<DVec<u8>> as *const u8;
                        let _ = v.into_raw_parts();
                        copy_nonoverlapping(orig_addr, addr, siz);
                        return_vals = Vec::from_raw_parts(addr, siz, cap);
                    }
                    return_vals

                } else {
                    panic!("Unsupported subsub type id: {}", subsub_typeid);
                }
            }
            else if sub_typeid == 17 {
                let p = unsafe {
                    transmute::<(usize, usize), Box<dyn Future<Output = DVec<i32>> + Send>>(new_ptr)
                };
                let f = Pin::from(p);
                let mut v = f.await;
                let siz = std::mem::size_of_val(&v);
                let mut return_vals = Vec::with_capacity(siz);
                unsafe {
                    let (addr, len, cap) = return_vals.into_raw_parts();
                    let orig_addr = &v as *const DVec<i32> as *const u8;
                    let _ = v.into_raw_parts();
                    copy_nonoverlapping(orig_addr, addr, siz);
                    return_vals = Vec::from_raw_parts(addr, siz, cap);
                }
                return_vals
            } 
            else if sub_typeid == 4 {
                let p = unsafe {
                    transmute::<(usize, usize), Box<dyn Future<Output = DVec<Chunk>> + Send>>(new_ptr)
                };
                let f = Pin::from(p);
                let mut v = f.await;
                let siz = std::mem::size_of_val(&v);
                let mut return_vals = Vec::with_capacity(siz);
                unsafe {
                    let (addr, len, cap) = return_vals.into_raw_parts();
                    let orig_addr = &v as *const DVec<Chunk> as *const u8;
                    let _ = v.into_raw_parts();
                    copy_nonoverlapping(orig_addr, addr, siz);
                    return_vals = Vec::from_raw_parts(addr, siz, cap);
                }
                return_vals
            }
            else if (sub_typeid & 0xFF) == 16 {
                let tuple_len = (sub_typeid >> 8);
                if tuple_len == 2 {
                    let p = unsafe {
                        transmute::<(usize, usize), Box<dyn Future<Output = (DVec<usize>, DVec<usize>)> + Send>>(new_ptr)
                    };
                    let f = Pin::from(p);
                    let mut v = f.await;
                    let siz = std::mem::size_of_val(&v);
                    let mut return_vals = Vec::with_capacity(siz);
                    unsafe {
                        let (addr, len, cap) = return_vals.into_raw_parts();
                        let orig_addr = &v as *const (DVec<usize>, DVec<usize>) as *const u8;
                        let (v1, v2) = v;
                        let _ = v1.into_raw_parts();
                        let _ = v2.into_raw_parts();
                        copy_nonoverlapping(orig_addr, addr, siz);
                        return_vals = Vec::from_raw_parts(addr, siz, cap);
                    }
                    return_vals
                }
                else if tuple_len == 3 {
                    let p = unsafe {
                        transmute::<(usize, usize), Box<dyn Future<Output = (DVec<usize>, DVec<usize>, DVec<usize>)> + Send>>(new_ptr)
                    };
                    let f = Pin::from(p);
                    let mut v = f.await;
                    let siz = std::mem::size_of_val(&v);
                    let mut return_vals = Vec::with_capacity(siz);
                    unsafe {
                        let (addr, len, cap) = return_vals.into_raw_parts();
                        let orig_addr = &v as *const (DVec<usize>, DVec<usize>, DVec<usize>) as *const u8;
                        let (v1, v2, v3) = v;
                        let _ = v1.into_raw_parts();
                        let _ = v2.into_raw_parts();
                        let _ = v3.into_raw_parts();
                        copy_nonoverlapping(orig_addr, addr, siz);
                        return_vals = Vec::from_raw_parts(addr, siz, cap);
                    }
                    return_vals
                } else {
                    panic!("Unsupported tuple length: {}", tuple_len)

                }
                
            } else {
                panic!("Unsupported sub type id: {}", sub_typeid);
            }
        } else if (typeid & 0xFF) == 2 {
            let sub_typeid = typeid >> 8;
            if sub_typeid == 21 {
                let p = unsafe {
                    transmute::<(usize, usize), Box<dyn Future<Output = DVecRef<'static, Image>> + Send>>(new_ptr)
                };
                let f = Pin::from(p);
                let mut v = f.await;
                dprintln!("media images v.len(): {} copy_exists: {}", v.len(), v.copy_exists);
                v.drop_copy();
                dprintln!("media images after drop copy v.len(): {} {}", v.len(), v.copy_exists);
                let siz = std::mem::size_of_val(&v);
                let mut return_vals = Vec::with_capacity(siz);
                unsafe {
                    let (addr, len, cap) = return_vals.into_raw_parts();
                    let orig_addr = &v as *const DVecRef<'static, Image> as *const u8;
                    copy_nonoverlapping(orig_addr, addr, siz);
                    return_vals = Vec::from_raw_parts(addr, siz, cap);
                }
                
                dprintln!("media images after copy v.len(): {}", v.len());
                return_vals
            } else {
                panic!("Unsupported sub type id: {}", sub_typeid);
            }
        }
         else if (typeid & 0xFF) == 64 {
            let p = unsafe {
                transmute::<(usize, usize), Box<dyn Future<Output = ()> + Send>>(new_ptr)
            };
            let f = Pin::from(p);
            let mut v = f.await;
            let siz = std::mem::size_of_val(&v);
            let mut return_vals = Vec::with_capacity(siz);
            unsafe {
                let (addr, len, cap) = return_vals.into_raw_parts();
                let orig_addr = &v as *const () as *const u8;
                copy_nonoverlapping(orig_addr, addr, siz);
                return_vals = Vec::from_raw_parts(addr, siz, cap);
            }
            return_vals
        } else if (typeid & 0xFF) == 16 {
            let p = unsafe {
                transmute::<(usize, usize), Box<dyn Future<Output = usize> + Send>>(new_ptr)
            };
            let f = Pin::from(p);
            let mut v = f.await;
            let siz = std::mem::size_of_val(&v);
            let mut return_vals = Vec::with_capacity(siz);
            unsafe {
                let (addr, len, cap) = return_vals.into_raw_parts();
                let orig_addr = &v as *const usize as *const u8;
                copy_nonoverlapping(orig_addr, addr, siz);
                return_vals = Vec::from_raw_parts(addr, siz, cap);
            }
            return_vals
        } else {
            panic!("Unsupported type id: {}", typeid);
        }
    }
}

pub async fn run_server(server_addr: SocketAddr) {
    drun_server!(server_addr, DRustServer);
}

pub static mut DCLIENTS: Option<Vec<Arc<DRustWorldClient>>> = None;
pub fn get_dclient(server_idx: usize) -> Arc<DRustWorldClient> {
    unsafe { Arc::clone(DCLIENTS.as_ref().unwrap().get(server_idx).unwrap()) }
}
