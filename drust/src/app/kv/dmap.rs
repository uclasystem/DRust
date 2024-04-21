
use arr_macro::arr;
use clap::Parser;
use crossbeam::atomic;
use futures::{future, prelude::*, channel::oneshot::channel};
use good_memory_allocator::SpinLockedAllocator;
use num::integer::Roots;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use serde::{Deserialize, Serialize};
use core::panic;
use std::{sync::Mutex, intrinsics, alloc::{Allocator, Layout}, slice::SliceIndex, ptr::NonNull};
use std::time::{Instant, SystemTime};
use std::{fmt, sync::atomic::AtomicBool};
use std::{io, mem, net::SocketAddr, ptr, sync::Arc, thread, time::Duration};
use tokio::{runtime::Runtime, sync::mpsc};
use dashmap::DashMap;

use crate::drust_std::{collections::dvec::{DVec, DVecRef}, sync::dmutex::DMutex};

use super::{entry::*, conf::*};


pub struct KVStore(DVec<DMutex<GlobalEntry>>);

impl KVStore {
    pub fn new() -> DVec<DMutex<GlobalEntry>> {
        let mut store = DVec::with_capacity(BUCKET_NUM);
        for _ in 0..BUCKET_NUM {
            store.push(DMutex::new(GlobalEntry {key: 0, value: [0; 32],}));
        }
        store
    }
}


pub fn get(map: &DVecRef<'_, DMutex<GlobalEntry>>, key: usize) -> [u8; 32] {
    let map_ref = map.as_ref();
    let bucket_id = bucket(key);
    let m = map_ref.get(bucket_id).unwrap();
    let value_ref = m.lock();
    let v = value_ref.value;
    m.unlock(value_ref);
    v
}

pub async fn put(map: &DVecRef<'_, DMutex<GlobalEntry>>, key: usize, value: [u8; 32]) {
    
    let map_ref = map.as_ref();
    let bucket_id = bucket(key);
    let m = map_ref.get(bucket_id).unwrap();
    let value_ref = m.lock();
    value_ref.key = key;
    value_ref.value = value;
    m.unlock(value_ref);
}