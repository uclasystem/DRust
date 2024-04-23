use futures::{future, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    cmp, fs::File, io::{self, Write}, net::SocketAddr, sync::Arc, time::{Duration, SystemTime}
};
use tokio::{
    runtime::Runtime,
    time::{self, Instant},
};

use super::super::{frame, prelude::DataType};
use super::super::prelude::*;
use super::utils::*;

use crate::drust_std::utils::*;
use crate::conf::*;

async fn groupby_work(
    x: &DataFrame,
    keys_ids: Vec<String>,
    values: Vec<(String, AggType)>,
) -> DataFrame {
    let groupby = x.groupby(keys_ids).await.unwrap();
    println!("groups len: {:?}", groupby.groups().0.len());
    let frame = std::thread::scope(|s| {
        let keys_handle = s.spawn(|| Runtime::new().unwrap().block_on(groupby.keys()));
        // let mut frame_vec = keys_handle.join().unwrap();
        // println!("after keys");
        let mut val_handles = vec![];
        for (series_name, typ) in &values {
            match *typ {
                AggType::Sum => {
                    val_handles.push(s.spawn(|| {
                        Runtime::new()
                            .unwrap()
                            .block_on(groupby.sum_series(series_name))
                            .unwrap()
                    }));
                }
                AggType::Min => {
                    val_handles.push(s.spawn(|| {
                        Runtime::new()
                            .unwrap()
                            .block_on(groupby.min_series(series_name))
                            .unwrap()
                    }));
                }
                _ => {
                    panic!("not implemented");
                }
            }
        }
        let mut frame_vec = keys_handle.join().unwrap();
        for val_handle in val_handles {
            let v = val_handle.join().unwrap();
            frame_vec.push(v);
            println!("after val");
        }
        frame_vec
    });
    let mut frame = DataFrame::new(frame).unwrap();
    frame
}

pub async fn h2oai_groupby_benchmark(dataset_size: DSize) {
    unsafe{
        COMPUTES = Some(ResourceManager::new(NUM_SERVERS));
    }

    let (f, line_cnt) = match dataset_size {
        DSize::Small => ("group.csv", 10),
        DSize::Medium => ("G1_1e7_1e2_0_0.csv", 10000000),
        DSize::Large => ("G1_1e8_1e2_0_0.csv", 100000000),
        DSize::Huge => ("G1_1e9_1e2_0_0.csv", 1000000000),
    };
    let mut x = read_csv_from_file(
        f,
        vec![
            DataType::UInt32,
            DataType::UInt32,
            DataType::UInt32,
            DataType::UInt32,
            DataType::UInt32,
            DataType::UInt32,
            DataType::Int32,
            DataType::Int32,
            DataType::Float64,
        ],
        line_cnt,
    )
    .await
    .expect("frame loading error");
    println!("{:?}", x.width());
    println!("{:?}", x.n_chunks());
    print(&mut x).await;

    let file_name = format!(
        "{}/DRust_home/logs/dataframe_drust_{}.txt", dirs::home_dir().unwrap().display(), NUM_SERVERS
    );
    let mut wrt_file = File::create(file_name).expect("file");

    let now = Instant::now();

    let frames = std::thread::scope(|s| {
        let frame10_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec![
                    "id1".to_string(),
                    "id2".to_string(),
                    "id3".to_string(),
                    "id4".to_string(),
                    "id5".to_string(),
                    "id6".to_string(),
                ],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame10_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec![
                    "id1".to_string(),
                    "id2".to_string(),
                    "id3".to_string(),
                    "id4".to_string(),
                    "id5".to_string(),
                    "id6".to_string(),
                ],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        std::thread::sleep(Duration::from_millis(100));

        let frame1_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame2_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string(), "id2".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame3_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame4_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame5_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame6_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string(), "id5".to_string()],
                vec![
                    ("v3".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Min),
                ],
            ))
        });
        let frame7_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Min),
                    ("v2".to_string(), AggType::Min),
                ],
            ))
        });
        let frame8_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![("v3".to_string(), AggType::Min)],
            ))
        });
        let frame9_handle = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id2".to_string(), "id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v2".to_string(), AggType::Sum),
                ],
            ))
        });

        let frame1_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame2_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string(), "id2".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame3_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame4_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame5_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame6_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string(), "id5".to_string()],
                vec![
                    ("v3".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Min),
                ],
            ))
        });
        let frame7_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Min),
                    ("v2".to_string(), AggType::Min),
                ],
            ))
        });
        let frame8_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![("v3".to_string(), AggType::Min)],
            ))
        });
        let frame9_handle2 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id2".to_string(), "id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v2".to_string(), AggType::Sum),
                ],
            ))
        });

        let frame1_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame2_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id1".to_string(), "id2".to_string()],
                vec![("v1".to_string(), AggType::Sum)],
            ))
        });
        let frame3_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame4_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame5_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Sum),
                ],
            ))
        });
        let frame6_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id4".to_string(), "id5".to_string()],
                vec![
                    ("v3".to_string(), AggType::Sum),
                    ("v3".to_string(), AggType::Min),
                ],
            ))
        });
        let frame7_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id3".to_string()],
                vec![
                    ("v1".to_string(), AggType::Min),
                    ("v2".to_string(), AggType::Min),
                ],
            ))
        });
        let frame8_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id6".to_string()],
                vec![("v3".to_string(), AggType::Min)],
            ))
        });
        let frame9_handle3 = s.spawn(|| {
            Runtime::new().unwrap().block_on(groupby_work(
                &x,
                vec!["id2".to_string(), "id4".to_string()],
                vec![
                    ("v1".to_string(), AggType::Sum),
                    ("v2".to_string(), AggType::Sum),
                ],
            ))
        });
        
        let frame1 = frame1_handle.join().unwrap();
        let frame2 = frame2_handle.join().unwrap();
        let frame3 = frame3_handle.join().unwrap();
        let frame4 = frame4_handle.join().unwrap();
        let frame5 = frame5_handle.join().unwrap();
        let frame6 = frame6_handle.join().unwrap();
        let frame7 = frame7_handle.join().unwrap();
        let frame8 = frame8_handle.join().unwrap();
        let frame9 = frame9_handle.join().unwrap();
        let frame1_2 = frame1_handle2.join().unwrap();
        let frame2_2 = frame2_handle2.join().unwrap();
        let frame3_2 = frame3_handle2.join().unwrap();
        let frame4_2 = frame4_handle2.join().unwrap();
        let frame5_2 = frame5_handle2.join().unwrap();
        let frame6_2 = frame6_handle2.join().unwrap();
        let frame7_2 = frame7_handle2.join().unwrap();
        let frame8_2 = frame8_handle2.join().unwrap();
        let frame9_2 = frame9_handle2.join().unwrap();
        let frame1_3 = frame1_handle3.join().unwrap();
        let frame2_3 = frame2_handle3.join().unwrap();
        let frame3_3 = frame3_handle3.join().unwrap();
        let frame4_3 = frame4_handle3.join().unwrap();
        let frame5_3 = frame5_handle3.join().unwrap();
        let frame6_3 = frame6_handle3.join().unwrap();
        let frame7_3 = frame7_handle3.join().unwrap();
        let frame8_3 = frame8_handle3.join().unwrap();
        let frame9_3 = frame9_handle3.join().unwrap();

        let frame10 = frame10_handle.join().unwrap();
        let frame10_2 = frame10_handle2.join().unwrap();

        vec![
            frame1, frame2, frame3, frame4, frame5, frame6, frame7, frame8, frame9, frame10,
            frame1_2, frame2_2, frame3_2, frame4_2, frame5_2, frame6_2, frame7_2, frame8_2,
            frame9_2, frame10_2, frame1_3, frame2_3, frame3_3, frame4_3, frame5_3, frame6_3,
            frame7_3, frame8_3, frame9_3,
        ]
    });

    let duration = now.elapsed().as_millis();
    for mut frame in frames {
        print(&mut frame).await;
    }

    println!("query {:?}, {}ms", dataset_size, duration);
    writeln!(wrt_file, "{}", duration as f64 / 1000.0).expect("write");
}
