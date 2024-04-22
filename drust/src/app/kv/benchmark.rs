use std::{fs::File, sync::Arc, io::Write};

use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use tokio::{runtime::Runtime, task::JoinHandle};

use super::{dmap::KVStore, dmap::*, entry::GlobalEntry, conf::{bucket, READ_RATIO, UNIT_BUCKET_NUM, UNIT_THREAD_BUCKET_NUM, THREAD_NUM}};


use crate::{conf::{GLOBAL_HEAP_START, NUM_SERVERS, SERVER_INDEX, WORKER_UNIT_SIZE}, drust_std::{collections::dvec::DVecRef, sync::dmutex::DMutex, thread::dspawn_to}};


static mut KEYS: Option<Vec<Vec<(usize, i32)>>> = None;

pub async fn populate(map: DVecRef<'_, DMutex<GlobalEntry>>) {
    let v = ['x' as u8; 32];
    let csv_file = format!("{}/DRust_home/dataset/dht/zipf/gam_data_0.99_100000000_{}_{}.csv", dirs::home_dir().unwrap().display(), NUM_SERVERS, unsafe{SERVER_INDEX % NUM_SERVERS});
    let mut rdr = csv::Reader::from_path(csv_file).unwrap();
    let mut cnt = 0;
    let popstart = tokio::time::Instant::now();
    for result in rdr.records() {
        if cnt % 1000000 == 0 {
            println!("Populate {} keys", cnt);
        }
        let record = result.unwrap();
        let key: usize = record[0].parse().unwrap();
        put(&map, key, v).await;
        cnt += 1;
    }

    cnt = 0;
    let csv_file = format!("{}/DRust_home/dataset/dht/zipf/gam_data_0.99_100000000_{}_{}.csv", dirs::home_dir().unwrap().display(), NUM_SERVERS, unsafe{(SERVER_INDEX + 1) % NUM_SERVERS});
    rdr = csv::Reader::from_path(csv_file).unwrap();
    let mut rng = thread_rng();
    let range = Uniform::from(0..100);
    let mut keys_vec = vec![];
    for i in 0..THREAD_NUM {
        keys_vec.push(vec![]);
    }
    
    for result in rdr.records() {
        let record = result.unwrap();
        let key: usize = record[0].parse().unwrap();
        let r = range.sample(&mut rng);
        let bucket_id = bucket(key);
        let bucket_s_offset = bucket_id % UNIT_BUCKET_NUM;
        let thread_id = bucket_s_offset / UNIT_THREAD_BUCKET_NUM;
        keys_vec[thread_id].push((key, r));
        cnt += 1;
    }
    unsafe{KEYS = Some(keys_vec);}
}

pub async fn benchmark(map: DVecRef<'_, DMutex<GlobalEntry>>) {
    let mut cnt = 0;
    let v = ['x' as u8; 32];
    let start = tokio::time::Instant::now();
    std::thread::scope(|s| {
        let mut handles = vec![];
        for keys in unsafe{KEYS.as_ref().unwrap()} {
            let map_ref = &map;
            let handle = s.spawn(move || {

                Runtime::new().unwrap().block_on(async {
                    let thread_local_start = tokio::time::Instant::now();
                    let mut thread_local_cnt = 0;
                    for (key, r) in keys {
                        if *r < READ_RATIO {
                            let getv = get(map_ref, *key);
                            if getv != v {
                                panic!("Wrong value");
                            }
                        } else {
                            put(map_ref, *key, v).await;
                        }
                        thread_local_cnt += 1;
                    }
                    let duration = thread_local_start.elapsed();
                    println!("Thread Local Elapsed Time: {:?}, throughput: {:?}", duration, thread_local_cnt as f64 / duration.as_secs_f64());
                })

            });
            handles.push(handle);
            
        }
        for handle in handles {
            handle.join().unwrap();
        }
        
    });
    let time = start.elapsed();
    println!("Local Elapsed Time: {:?}", time);


    // let mut handles = vec![];
    // for keys in unsafe{KEYS.as_ref().unwrap()} {
    //     let arc_map_ref = arc_map.clone();
    //     let handle = tokio::spawn(async move {
    //         let thread_local_start = tokio::time::Instant::now();
    //         let mut thread_local_cnt = 0;
    //         for (key, r) in keys {
    //             if *r < READ_RATIO {
    //                 let getv = get(&*arc_map_ref, *key);
    //                 if getv != v {
    //                     panic!("Wrong value");
    //                 }
    //             } else {
    //                 put(&*arc_map_ref, *key, v).await;
    //             }
    //             thread_local_cnt += 1;
    //         }
    //         let duration = thread_local_start.elapsed();
    //         println!("Thread Local Elapsed Time: {:?}, throughput: {:?}", duration, thread_local_cnt as f64 / duration.as_secs_f64());
    //     });
    //     handles.push(handle);
    // }
    // for handle in handles {
    //     handle.await.unwrap();
    // }
    
    // let time = start.elapsed();
    // println!("Local Elapsed Time: {:?}", time);
    // println!("Local Throughput: {:?}", cnt as f64 / time.as_secs_f64());
}

// load column from file and return a Column struct
pub async fn zipf_bench() {
    let map = KVStore::new();

    let popstart = tokio::time::Instant::now();
    let mut handles = vec![];
    for i in 0..NUM_SERVERS {
        let map_ref = map.as_dref();
        let handle: JoinHandle<()> = dspawn_to(populate(map_ref), GLOBAL_HEAP_START + i * WORKER_UNIT_SIZE);
        handles.push(handle);
    }
    for handle in handles {
        handle.await;
    }
    println!("Populate Elapsed Time: {:?}", popstart.elapsed());
    


    
    let mut handles = vec![];
    let start = tokio::time::Instant::now();
    for i in 0..NUM_SERVERS {
        let map_ref = map.as_dref();
        let handle: JoinHandle<()> = dspawn_to(benchmark(map_ref), GLOBAL_HEAP_START + i * WORKER_UNIT_SIZE);
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await;
    }
    let time = start.elapsed();
    println!("Total Elapsed Time: {:?}", time);
    println!("Total Throughput: {:?}", 100000000 as f64 / time.as_secs_f64());   
    
    let file_name = format!(
        "{}/DRust_home/logs/kv_drust_{}.txt", dirs::home_dir().unwrap().display(), NUM_SERVERS
    );
    let mut wrt_file = File::create(file_name).expect("file");
    let milli_seconds = time.as_millis();
    writeln!(wrt_file, "{}", milli_seconds as f64 / 1000.0).expect("write");
}
