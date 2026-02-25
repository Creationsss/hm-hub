use anyhow::{bail, Result};

use crate::consts::{CRC_OFFSET, PACKET_SIZE, PAYLOAD_SIZE};

pub struct Packet {
	pub buf: [u8; PACKET_SIZE],
}

impl Packet {
	pub fn new(cmd_id: u8, payload: &[u8]) -> Result<Self> {
		if payload.len() > PAYLOAD_SIZE {
			bail!("payload too large: {} > {}", payload.len(), PAYLOAD_SIZE);
		}
		let mut buf = [0u8; PACKET_SIZE];
		buf[0] = cmd_id;
		buf[1..1 + payload.len()].copy_from_slice(payload);
		let crc = crc32fast::hash(&buf[..CRC_OFFSET]);
		buf[CRC_OFFSET..].copy_from_slice(&crc.to_le_bytes());
		Ok(Packet { buf })
	}

	pub fn from_bytes(buf: [u8; PACKET_SIZE]) -> Result<Self> {
		let expected = crc32fast::hash(&buf[..CRC_OFFSET]);
		let actual = u32::from_le_bytes([
			buf[CRC_OFFSET],
			buf[CRC_OFFSET + 1],
			buf[CRC_OFFSET + 2],
			buf[CRC_OFFSET + 3],
		]);
		if expected != actual {
			bail!("CRC mismatch: expected {expected:#x}, got {actual:#x}");
		}
		Ok(Packet { buf })
	}

	pub fn cmd_id(&self) -> u8 {
		self.buf[0]
	}

	pub fn payload(&self) -> &[u8] {
		&self.buf[1..CRC_OFFSET]
	}

	pub fn send(&self, port: &mut dyn serialport::SerialPort) -> Result<()> {
		port.write_all(&self.buf)?;
		port.flush()?;
		Ok(())
	}
}
