use std::{cmp, fs::File, io::Write, sync::Arc, time::SystemTime};

use arr_macro::arr;
use super::{conf::*, media::utils::decode_and_extract};

use crate::{ app::socialnet::{compose, media::{init_media_storage, MEDIA}}, conf::{NUM_SERVERS, SERVER_INDEX}, drust_std::{collections::dstring::DString, connect::dsafepoint::set_ready, thread::{dspawn_to_relaxed, dspawn_to_strictly}, utils::{SimpleResourceManager, SIMPLE_COMPUTES}}};


pub async fn socialnet_benchmark() {
  unsafe {
    let mut simple_computes = Vec::new();
    for i in 0..NUM_SERVERS {
      simple_computes.push(SimpleResourceManager::new(THREAD_NUM));
    }
    SIMPLE_COMPUTES = Some(simple_computes);
  }
  for i in VIDEO_STORAGE_SERVER_START..(VIDEO_STORAGE_SERVER_START + VIDEO_STORAGE_SERVER_NUM) {
    let result: () = dspawn_to_relaxed(init_media_storage(), i).await.unwrap();
  }


  let start = SystemTime::now();
  let mut id = 0;
  let total_reqs = 18432;
  let unit_reqs = cmp::min(256 * NUM_SERVERS, 1024);
  for i in 0..(total_reqs / unit_reqs){
      let mut handles = Vec::new();
      for id in (i * unit_reqs)..(i * unit_reqs + unit_reqs) {
          let text_str = "0 1467810672 Mon Apr 06 22:19:49 PDT 2009 NO_QUERY scotthamilton is upset that he can't update his Facebook by texting it... and might cry as a result  School today also. Blah!";
          let len = text_str.len();
          let mut text = DString::with_capacity(len);
          text.as_mut().extend(text_str.bytes());
          handles.push(tokio::spawn(async move {
            compose::compose_post(id, 0, text, id, 0).await;
          }));
      }
      println!("i {}", i);
      for handle in handles {
          handle.await.unwrap();
      }
  }
  let time = start.elapsed().unwrap();
  println!("Elapsed Time: {:?}", time);
  
  let file_name = format!(
    "{}/DRust_home/logs/sn_drust_{}.txt", dirs::home_dir().unwrap().display(), NUM_SERVERS
  );
  let mut wrt_file = File::create(file_name).expect("file");
  let milli_seconds = time.as_millis();
  writeln!(wrt_file, "{}", milli_seconds as f64 / 1000.0).expect("write");
  
}