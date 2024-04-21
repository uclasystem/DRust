pub mod utils;
pub mod watermark;

use rand::{distributions::{Distribution, Uniform}, thread_rng};

use crate::{conf::SERVER_INDEX, dassert, dprintln, drust_std::{collections::dvec::{DVec, DVecRef}, primitives::dbox::DBox, thread::{dspawn_to, dspawn_to_relaxed, dspawn_to_strictly}}};

use watermark::do_watermark;
use super::conf::*;

#[derive(Default, Clone)]
pub struct Image {
  pub width: u32,
  pub height: u32,
  pub pixels: DVec<u8>,
}

pub static mut MEDIA: Option<Vec<DVec<Image>>> = None;

pub async fn init_media_storage() {
  unsafe {
    dassert!(SERVER_INDEX >= VIDEO_STORAGE_SERVER_START && SERVER_INDEX < VIDEO_STORAGE_SERVER_START + VIDEO_STORAGE_SERVER_NUM, "media storage server index error");
    let media = utils::decode_and_extract().unwrap();
    let mut medias = Vec::with_capacity(4096);
    for _ in 0..4096 {
      medias.push(media.clone());
    }
    MEDIA = Some(medias);
    dprintln!("video server {} started", SERVER_INDEX);
  }
}


pub async fn get_media(media_id: usize) -> DVecRef<'static, Image> {
  
  let medias = unsafe{MEDIA.as_ref().unwrap()};
  let index = media_id % medias.len();
  let r = medias[index].as_dref();
  dprintln!("get_media: media_id: {}, index: {}", media_id, index);
  r
}

async fn mock_store(images: DVec<Image>) {
  let images_ref = images.as_ref();
  for image in images_ref {
    let pixels_ref = image.pixels.as_ref();
    let pixels = pixels_ref.to_vec();
    dprintln!("pixels len: {}", pixels.len());
  }
}

pub async fn store_media(images: DVec<Image>) -> usize {
  mock_store(images);
  let mut id = 0;
  let mut rng = thread_rng();
  let range = Uniform::new(0, 1000000000000000000);
  range.sample(&mut rng)
}

pub async fn media_service(media_id: usize) -> usize {
  let media_storage_id = media_id % VIDEO_STORAGE_SERVER_NUM + VIDEO_STORAGE_SERVER_START;

  dprintln!("media_service: media_id: {}, media_storage_id: {}", media_id, media_storage_id);

  let media: DVecRef<'_, Image> = dspawn_to_relaxed(get_media(media_id), media_storage_id).await.unwrap();

  dprintln!("media_service: media.len: {}", media.len());

  let len = media.len();
  let mut watermarked_images = DVec::with_capacity(len);
  let mut index = 0;
  while index < len {
      let mut image_data = media[index].clone();
      dprintln!("image_data len: {}", image_data.pixels.len());
      do_watermark(&mut image_data);
      dprintln!("watermarked image_data: {}", image_data.pixels.len());
      watermarked_images.push(image_data);
      index += 1;
  }
  drop(media);
  dprintln!("watermarked_images len: {}", watermarked_images.len());
  let new_media_id = dspawn_to_relaxed(store_media(watermarked_images), media_storage_id).await.unwrap();
  dprintln!("new_media_id: {}", new_media_id);
  new_media_id
}

