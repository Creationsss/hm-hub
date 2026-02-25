use anyhow::Result;

use crate::consts::{CHUNK_DATA_SIZE, PAYLOAD_SIZE};
use crate::types::Packet;

pub fn encode_chunked(cmd_id: u8, sub_cmd: u8, data: &[u8]) -> Result<Vec<Packet>> {
	let crc = crc32fast::hash(data);
	let mut full_data = Vec::with_capacity(data.len() + 4);
	full_data.extend_from_slice(data);
	full_data.extend_from_slice(&crc.to_le_bytes());

	let total_len = full_data.len();
	let total_chunks = total_len.div_ceil(CHUNK_DATA_SIZE);
	let mut packets = Vec::with_capacity(total_chunks);
	let mut remaining = total_len;

	for i in 0..total_chunks {
		let chunk_size = remaining.min(CHUNK_DATA_SIZE);
		let offset = total_len - remaining;

		let mut payload = [0u8; PAYLOAD_SIZE];
		payload[0] = sub_cmd;
		payload[1..3].copy_from_slice(&(total_chunks as u16).to_le_bytes());
		payload[3..5].copy_from_slice(&(i as u16).to_le_bytes());
		payload[5..7].copy_from_slice(&(chunk_size as u16).to_le_bytes());
		payload[7..7 + chunk_size].copy_from_slice(&full_data[offset..offset + chunk_size]);

		packets.push(Packet::new(cmd_id, &payload)?);
		remaining -= chunk_size;
	}

	Ok(packets)
}
