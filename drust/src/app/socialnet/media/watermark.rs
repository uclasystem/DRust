use image::{Rgb, RgbImage};
use imageproc::{map::map_colors, drawing::{draw_text_mut, text_size}};
use imageproc::pixelops::weighted_sum;
use rusttype::{Font, Scale};

use super::Image;


/// Tint a grayscale value with the given color.
/// Midtones are tinted most heavily.
fn tint(gray: Rgb<u8>, color: Rgb<u8>) -> Rgb<u8> {
  let dist_from_mid = ((gray[0] as f32 - 128f32).abs()) / 255f32;
  let scale_factor = 1f32 - 4f32 * dist_from_mid.powi(2);
  weighted_sum(Rgb([gray[0]; 3]), color, 1.0, scale_factor)
}

/// Linearly interpolates between low and mid colors for pixel intensities less than
/// half of maximum brightness and between mid and high for those above.
fn color_gradient(gray: Rgb<u8>, low: Rgb<u8>, mid: Rgb<u8>, high: Rgb<u8>) -> Rgb<u8> {
  let fraction = gray[0] as f32 / 255f32;
  let (lower, upper, offset) = if fraction < 0.5 {
      (low, mid, 0.0)
  } else {
      (mid, high, 0.5)
  };
  let right_weight = 2.0 * (fraction - offset);
  let left_weight = 1.0 - right_weight;
  weighted_sum(lower, upper, left_weight, right_weight)
}


pub fn do_watermark(image: &mut Image) {
  let image_ref = image.pixels.as_mut();
  let image_vec = image_ref.to_vec();
  let inner_image: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_raw(image.width, image.height, image_vec).unwrap();
  let blue = Rgb([0u8, 0u8, 255u8]);
  // // Apply the color tint to every pixel in the grayscale image, producing a image::RgbImage
  // let tinted = map_colors(&image, |pix| tint(pix, blue));
  // Apply color gradient to each image pixel
  let black = Rgb([0u8, 0u8, 0u8]);
  let red = Rgb([255u8, 0u8, 0u8]);
  let yellow = Rgb([255u8, 255u8, 0u8]);

  let inner_image = map_colors(&inner_image, |pix| tint(pix, blue));
  let mut inner_image = map_colors(&inner_image, |pix| color_gradient(pix, black, red, yellow));

  let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
  let font = Font::try_from_vec(font).unwrap();

  let height = 20.0;
  let scale = Scale {
      x: height * 2.0,
      y: height,
  };
  let text = "Hello, world!";
  draw_text_mut(&mut inner_image, Rgb([0u8, 0u8, 255u8]), 0, 0, scale, &font, text);

  let text = "DRust!";
  draw_text_mut(&mut inner_image, Rgb([0u8, 0u8, 255u8]), 0, 0, scale, &font, text);
  
  let inner_image = map_colors(&inner_image, |pix| tint(pix, blue));
  let inner_image = map_colors(&inner_image, |pix| color_gradient(pix, black, red, yellow));

  // Convert gradient to a vector of bytes
  let watermark_vec = inner_image.into_raw();
  unsafe{std::ptr::copy_nonoverlapping(watermark_vec.as_ptr(), image_ref.as_mut_ptr(), image.width as usize * image.height as usize * 3 as usize)};
}
