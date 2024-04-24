use std::time::{SystemTime, UNIX_EPOCH};

use ffmpeg_next::ffi::dprintf;
use tokio::task::JoinHandle;

use crate::{conf::NUM_SERVERS, dprintln, drust_std::{collections::{dstring::DString, dvec::DVec}, thread::{dspawn, dspawn_to, dspawn_to_relaxed, dspawn_to_strictly}}};

use super::{conf::*, media::media_service, post::Post, storage::store_post, text::text_service, uniqueid};


pub async fn compose_post(req_id: usize, user_id: usize, mut text: DString, media_id: usize, post_type: u8) {
    let st = SystemTime::now();
    // let unique_id: usize = dspawn_to_strictly(uniqueid::unique_id(), UNIQUE_ID_SERVER_ID).await.unwrap();
    let unique_id = uniqueid::unique_id().await;
    dprintln!("unique_id: {}", unique_id);
    let mut media_id = (unique_id + 17) * 107 % 1000000007;
    let compute_server_idx = unique_id % NUM_SERVERS * 107 % NUM_SERVERS;
    dprintln!("compute_server_idx: {}", compute_server_idx);
    let media_service_handle: JoinHandle<usize> = dspawn_to_relaxed(media_service(media_id), compute_server_idx);
    let text_mut = text.as_dmut();
    let compute_server_idx2 = unique_id % NUM_SERVERS * 113 % NUM_SERVERS;
    let text_results: DVec<DString> = dspawn_to_relaxed(text_service(text_mut), compute_server_idx2).await.unwrap();
    // println!("text_results: {:?}", text.as_ref());
    let processed_media_id = media_service_handle.await.unwrap();
    dprintln!("before store: compose_post: req_id: {} media_id: {} processed_media_id: {}", req_id, media_id, processed_media_id);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let post = Post {
        post_id: unique_id,
        req_id,
        user_id,
        text,
        mentions: text_results,
        media_id,
        watermark_id: processed_media_id,
        timestamp: now,
        post_type,
    };
    let store_idx = compute_server_idx2;
    let store_result: () = dspawn_to_relaxed(store_post(post), store_idx).await.unwrap();
    dprintln!("compose_post: req_id: {} time: {}", req_id, st.elapsed().unwrap().as_millis());
}

