use std::sync::Once;

use dashmap::DashMap;

use crate::{dassert, dprintln};

use super::post::Post;


static mut MOCK_POST_STORAGE: Option<DashMap<usize, Vec<u8>>> = None;
static INIT_POST: Once = Once::new();


pub fn mock_store(post_id: usize, post_bytes: Vec<u8>) {
  unsafe {
    let post_storage = MOCK_POST_STORAGE.as_ref().unwrap();
    post_storage.insert(post_id, post_bytes);
  }
}

pub async fn store_post(post: Post) {
  INIT_POST.call_once(|| {
      unsafe{MOCK_POST_STORAGE = Some(DashMap::new());}
  });

  dprintln!("store_post: post_id: {}", post.post_id);

  let mut postbytes = Vec::new();
  postbytes.extend_from_slice(&post.post_id.to_be_bytes());
  postbytes.extend_from_slice(&post.req_id.to_be_bytes());
  postbytes.extend_from_slice(&post.media_id.to_be_bytes());
  postbytes.extend_from_slice(&post.watermark_id.to_be_bytes());
  postbytes.extend_from_slice(&post.timestamp.to_be_bytes());
  postbytes.push(post.post_type);
  let text_ref = post.text.as_ref();
  postbytes.extend_from_slice(&text_ref.len().to_be_bytes());
  postbytes.extend_from_slice(text_ref);
  postbytes.extend_from_slice(&post.mentions.len().to_be_bytes());
  let mentions_ref = post.mentions.as_ref();
  for mention in mentions_ref {
      let mention_ref = mention.as_ref();
      postbytes.extend_from_slice(&mention_ref.len().to_be_bytes());
      postbytes.extend_from_slice(mention_ref);
  }

  mock_store(post.post_id, postbytes);
  dprintln!("store_post: post_id: {} done", post.post_id);
}