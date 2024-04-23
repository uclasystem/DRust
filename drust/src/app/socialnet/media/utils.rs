use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video as ffmpegVideo;
use futures::SinkExt;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use super::super::conf::*;
use super::Image;
use crate::conf::*;
use crate::drust_std::collections::dvec::DVec;
use crate::{dassert, prelude::*};

pub fn decode_and_extract() -> Result<DVec<Image>, ffmpeg_next::Error> {
    ffmpeg_next::init().unwrap();
    let mut a = DVec::with_capacity(FRAME_NUM);
    let mut height = 0;
    let mut width = 0;
    let video_full_path = format!("{}/{}", dirs::home_dir().unwrap().display(), VIDEO_PATH);
    if let Ok(mut ictx) = input(&video_full_path) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
            Flags::BILINEAR,
        )?;
        height = decoder.height();
        width = decoder.width();
        let mut frame_index = 0;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg_next::decoder::Video| -> Result<(), ffmpeg_next::Error> {
                let mut decoded = ffmpegVideo::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = ffmpegVideo::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    // save_file(&rgb_frame, frame_index).unwrap();
                    if frame_index < 5 {
                        let mut frame_vec = DVec::with_capacity(FRAME_SIZE);
                        let mut frame_data = rgb_frame.data(0).to_vec();
                        frame_vec.as_mut().extend(&frame_data);
                        let image = Image { width: FRAME_WIDTH, height: FRAME_HEIGHT, pixels: frame_vec };
                        a.push(image);
                    }
                    frame_index += 1;
                }
                Ok(())
            };
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    // print!("{} ", a.len());

    Ok(a)
}


