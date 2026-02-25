use anyhow::{bail, Result};
use std::path::Path;

use crate::consts::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::types::Album;

pub fn rgba_to_rgb565(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
	let pixel_count = (width * height) as usize;
	let mut buf = vec![0u8; pixel_count * 2];
	for i in 0..pixel_count {
		let r = rgba[4 * i] as u16;
		let g = rgba[4 * i + 1] as u16;
		let b = rgba[4 * i + 2] as u16;
		let pixel = ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | ((b & 0xF8) >> 3);
		buf[2 * i] = (pixel >> 8) as u8;
		buf[2 * i + 1] = (pixel & 0xFF) as u8;
	}
	buf
}

pub fn load_image(path: &Path, crop: bool) -> Result<Album> {
	let ext = path
		.extension()
		.and_then(|e| e.to_str())
		.unwrap_or("")
		.to_lowercase();

	match ext.as_str() {
		"gif" => load_gif(path, crop),
		"png" | "jpg" | "jpeg" | "bmp" | "webp" => load_static(path, crop),
		_ => bail!("unsupported image format: {ext}"),
	}
}

fn load_static(path: &Path, crop: bool) -> Result<Album> {
	let img = image::open(path)?;
	let resized = resize_image(&img, DISPLAY_WIDTH, DISPLAY_HEIGHT, crop);
	let rgba = resized.to_rgba8();
	let data = rgba_to_rgb565(rgba.as_raw(), DISPLAY_WIDTH, DISPLAY_HEIGHT);

	Ok(Album {
		frames: vec![data],
		delay_ms: 0,
	})
}

fn load_gif(path: &Path, crop: bool) -> Result<Album> {
	use gif::DecodeOptions;
	use std::fs::File;

	let file = File::open(path)?;
	let mut opts = DecodeOptions::new();
	opts.set_color_output(gif::ColorOutput::RGBA);
	let mut decoder = opts.read_info(file)?;

	let gif_width = decoder.width() as u32;
	let gif_height = decoder.height() as u32;

	let mut frames = Vec::new();
	let mut delay_ms = 0u16;
	let mut canvas = vec![0u8; (gif_width * gif_height * 4) as usize];

	while let Some(frame) = decoder.read_next_frame()? {
		if delay_ms == 0 && frame.delay > 0 {
			delay_ms = frame.delay * 10;
		}

		let fx = frame.left as u32;
		let fy = frame.top as u32;
		let fw = frame.width as u32;
		let fh = frame.height as u32;

		for y in 0..fh {
			for x in 0..fw {
				let src_idx = ((y * fw + x) * 4) as usize;
				let dst_x = fx + x;
				let dst_y = fy + y;
				if dst_x < gif_width && dst_y < gif_height {
					let dst_idx = ((dst_y * gif_width + dst_x) * 4) as usize;
					if frame.buffer[src_idx + 3] > 0 {
						canvas[dst_idx..dst_idx + 4]
							.copy_from_slice(&frame.buffer[src_idx..src_idx + 4]);
					}
				}
			}
		}

		let img = image::RgbaImage::from_raw(gif_width, gif_height, canvas.clone())
			.ok_or_else(|| anyhow::anyhow!("failed to create image from GIF frame"))?;
		let dyn_img = image::DynamicImage::from(img);
		let resized = resize_image(&dyn_img, DISPLAY_WIDTH, DISPLAY_HEIGHT, crop);
		let rgba = resized.to_rgba8();
		let data = rgba_to_rgb565(rgba.as_raw(), DISPLAY_WIDTH, DISPLAY_HEIGHT);
		frames.push(data);

		if frame.dispose == gif::DisposalMethod::Background {
			for y in 0..fh {
				for x in 0..fw {
					let dst_x = fx + x;
					let dst_y = fy + y;
					if dst_x < gif_width && dst_y < gif_height {
						let idx = ((dst_y * gif_width + dst_x) * 4) as usize;
						canvas[idx..idx + 4].copy_from_slice(&[0, 0, 0, 0]);
					}
				}
			}
		}
	}

	if frames.is_empty() {
		bail!("GIF has no frames");
	}

	Ok(Album { frames, delay_ms })
}

fn resize_image(
	img: &image::DynamicImage,
	target_w: u32,
	target_h: u32,
	crop: bool,
) -> image::DynamicImage {
	let src_w = img.width() as f64;
	let src_h = img.height() as f64;
	let target_aspect = target_w as f64 / target_h as f64;
	let src_aspect = src_w / src_h;

	if crop {
		let (crop_w, crop_h, crop_x, crop_y) = if src_aspect > target_aspect {
			let cw = src_h * target_aspect;
			let cx = (src_w - cw) / 2.0;
			(cw, src_h, cx, 0.0)
		} else {
			let ch = src_w / target_aspect;
			let cy = (src_h - ch) / 2.0;
			(src_w, ch, 0.0, cy)
		};

		let cropped = img.crop_imm(crop_x as u32, crop_y as u32, crop_w as u32, crop_h as u32);
		cropped.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
	} else {
		let fitted = img.resize(target_w, target_h, image::imageops::FilterType::Lanczos3);
		let mut canvas = image::RgbaImage::new(target_w, target_h);
		let offset_x = (target_w - fitted.width()) / 2;
		let offset_y = (target_h - fitted.height()) / 2;
		image::imageops::overlay(
			&mut canvas,
			&fitted.to_rgba8(),
			offset_x as i64,
			offset_y as i64,
		);
		image::DynamicImage::from(canvas)
	}
}
