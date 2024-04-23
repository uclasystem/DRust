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


pub async fn get(map: &DVecRef<'_, DMutex<GlobalEntry>>, key: usize) -> [u8; 32] {
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
    let mut value_ref = m.lock();
    value_ref.key = key;
    value_ref.value = value;
    m.unlock(value_ref);
}