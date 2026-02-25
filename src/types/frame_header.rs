use anyhow::{bail, Result};

use crate::consts::{FRAME_HEADER_SIZE, FRAME_MAGIC};

#[derive(Debug)]
pub struct FrameHeader {
	pub width: u16,
	pub height: u16,
	pub frame_count: u16,
	pub delay_ms: u16,
	pub data_offset: u32,
	pub data_length: u32,
	pub data_crc32: u32,
}

impl FrameHeader {
	pub fn write_to(&self, buf: &mut [u8]) {
		let mut pos = 0;
		buf[pos..pos + 4].copy_from_slice(&FRAME_MAGIC.to_le_bytes());
		pos += 4;
		buf[pos..pos + 2].copy_from_slice(&self.width.to_le_bytes());
		pos += 2;
		buf[pos..pos + 2].copy_from_slice(&self.height.to_le_bytes());
		pos += 2;
		buf[pos..pos + 2].copy_from_slice(&self.frame_count.to_le_bytes());
		pos += 2;
		buf[pos..pos + 2].copy_from_slice(&self.delay_ms.to_le_bytes());
		pos += 2;
		buf[pos..pos + 4].copy_from_slice(&self.data_offset.to_le_bytes());
		pos += 4;
		buf[pos..pos + 4].copy_from_slice(&self.data_length.to_le_bytes());
		pos += 4;
		buf[pos..pos + 4].copy_from_slice(&self.data_crc32.to_le_bytes());
		pos += 4;
		let hdr_crc = crc32fast::hash(&buf[pos - 24..pos]);
		buf[pos..pos + 4].copy_from_slice(&hdr_crc.to_le_bytes());
	}

	pub fn read_from(buf: &[u8]) -> Result<Option<Self>> {
		if buf.len() < FRAME_HEADER_SIZE {
			bail!("frame header too short");
		}
		let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
		if magic != FRAME_MAGIC {
			return Ok(None);
		}
		let width = u16::from_le_bytes([buf[4], buf[5]]);
		let height = u16::from_le_bytes([buf[6], buf[7]]);
		let frame_count = u16::from_le_bytes([buf[8], buf[9]]);
		let delay_ms = u16::from_le_bytes([buf[10], buf[11]]);
		let data_offset = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
		let data_length = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
		let data_crc32 = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
		let header_crc32 = u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);

		let expected_hdr_crc = crc32fast::hash(&buf[..24]);
		if expected_hdr_crc != header_crc32 {
			bail!("frame header CRC mismatch");
		}

		Ok(Some(FrameHeader {
			width,
			height,
			frame_count,
			delay_ms,
			data_offset,
			data_length,
			data_crc32,
		}))
	}
}
