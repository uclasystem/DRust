

use tokio::sync::Mutex;

use super::{entry::*, conf::*};


pub struct KVStore(Vec<Mutex<GlobalEntry>>);

impl KVStore {
    pub fn new() -> Vec<Mutex<GlobalEntry>> {
        let mut store = Vec::with_capacity(BUCKET_NUM);
        for _ in 0..BUCKET_NUM {
            store.push(Mutex::new(GlobalEntry {key: 0, value: [0; 32],}));
        }
        store
    }
}


pub async fn get(map: &Vec<Mutex<GlobalEntry>>, key: usize) -> [u8; 32] {
    let map_ref: &Vec<Mutex<GlobalEntry>> = map.as_ref();
    let bucket_id = bucket(key);
    let m = map_ref.get(bucket_id).unwrap();
    let value_ref = m.lock().await;
    let v = value_ref.value;
    drop(value_ref);
    v
}

pub async fn put(map: &Vec<Mutex<GlobalEntry>>, key: usize, value: [u8; 32]) {
    
    let map_ref: &Vec<Mutex<GlobalEntry>> = map.as_ref();
    let bucket_id = bucket(key);
    let m = map_ref.get(bucket_id).unwrap();
    let mut value_ref = m.lock().await;
    value_ref.key = key;
    value_ref.value = value;
    drop(value_ref);
}