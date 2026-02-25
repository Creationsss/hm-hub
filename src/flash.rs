use anyhow::{bail, Result};

use crate::consts::*;
use crate::types::{max_frames, Album, FrameHeader};

pub fn build_flash_buffer(albums: &[Album], flash_size: u32) -> Result<Vec<u8>> {
	let max = max_frames(flash_size);

	if albums.len() > MAX_FRAME_HEADERS {
		bail!("too many albums: {} > {}", albums.len(), MAX_FRAME_HEADERS);
	}

	let total_frames: usize = albums.iter().map(|a| a.frames.len()).sum();
	if total_frames > max {
		bail!("total frames ({total_frames}) exceeds device capacity ({max})");
	}

	let total_pixel_data: usize = albums
		.iter()
		.map(|a| a.frames.iter().map(|f| f.len()).sum::<usize>())
		.sum();
	let total_size = FLASH_HEADER_AREA + total_pixel_data;

	let mut buffer = vec![0u8; total_size];
	let mut data_offset = FLASH_HEADER_AREA;

	for (i, album) in albums.iter().enumerate() {
		let mut all_data = Vec::new();
		for frame in &album.frames {
			all_data.extend_from_slice(frame);
		}

		let data_crc = crc32fast::hash(&all_data);

		buffer[data_offset..data_offset + all_data.len()].copy_from_slice(&all_data);

		let header = FrameHeader {
			width: DISPLAY_WIDTH as u16,
			height: DISPLAY_HEIGHT as u16,
			frame_count: album.frames.len() as u16,
			delay_ms: album.delay_ms,
			data_offset: data_offset as u32,
			data_length: all_data.len() as u32,
			data_crc32: data_crc,
		};

		let hdr_start = i * FRAME_HEADER_SIZE;
		header.write_to(&mut buffer[hdr_start..hdr_start + FRAME_HEADER_SIZE]);

		data_offset += all_data.len();
	}

	Ok(buffer)
}
