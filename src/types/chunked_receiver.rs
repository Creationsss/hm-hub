use anyhow::{bail, Result};

pub struct ChunkedReceiver {
	buffer: Vec<u8>,
	total_chunks: usize,
	received: usize,
	initialized: bool,
}

impl ChunkedReceiver {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			total_chunks: 0,
			received: 0,
			initialized: false,
		}
	}

	pub fn feed(&mut self, payload: &[u8]) -> Result<Option<Vec<u8>>> {
		let _chunk_idx = payload[0] as usize;
		let total = payload[1] as usize;
		let chunk_len = u16::from_le_bytes([payload[2], payload[3]]) as usize;
		let chunk_data = &payload[4..4 + chunk_len];

		if !self.initialized {
			self.total_chunks = total;
			self.buffer = Vec::new();
			self.initialized = true;
		}

		self.buffer.extend_from_slice(chunk_data);
		self.received += 1;

		if self.received >= self.total_chunks {
			if self.buffer.len() < 4 {
				bail!("chunked data too small");
			}
			let data_len = self.buffer.len() - 4;
			let expected_crc = crc32fast::hash(&self.buffer[..data_len]);
			let actual_crc = u32::from_le_bytes([
				self.buffer[data_len],
				self.buffer[data_len + 1],
				self.buffer[data_len + 2],
				self.buffer[data_len + 3],
			]);
			if expected_crc != actual_crc {
				bail!("chunked CRC mismatch: expected {expected_crc:#x}, got {actual_crc:#x}");
			}
			self.buffer.truncate(data_len);
			Ok(Some(std::mem::take(&mut self.buffer)))
		} else {
			Ok(None)
		}
	}
}
