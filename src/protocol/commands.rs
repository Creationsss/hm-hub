use anyhow::{bail, Result};

use crate::consts::*;
use crate::types::{DeviceInfo, Packet, PowerStats};

pub fn build_handshake() -> Result<Packet> {
	Packet::new(CMD_HANDSHAKE, &[0; PAYLOAD_SIZE])
}

pub fn parse_handshake(packet: &Packet) -> Result<DeviceInfo> {
	if packet.cmd_id() != CMD_HANDSHAKE {
		bail!("expected handshake response, got cmd {}", packet.cmd_id());
	}
	let p = packet.payload();
	Ok(DeviceInfo {
		hw_id: u32::from_le_bytes([p[0], p[1], p[2], p[3]]),
		fw_ver: u32::from_le_bytes([p[4], p[5], p[6], p[7]]),
		flash_size: u32::from_le_bytes([p[8], p[9], p[10], p[11]]),
	})
}

pub fn build_config_read() -> Result<Packet> {
	let mut payload = [0u8; PAYLOAD_SIZE];
	payload[0] = 1;
	Packet::new(CMD_CONFIG, &payload)
}

pub fn build_flash_start(total_size: u32) -> Result<Packet> {
	let mut payload = [0u8; PAYLOAD_SIZE];
	payload[0] = 1;
	payload[1..5].copy_from_slice(&total_size.to_le_bytes());
	Packet::new(CMD_FLASH, &payload)
}

pub fn build_flash_data_response(offset: u32, length: u16, data: &[u8]) -> Result<Packet> {
	let mut payload = [0u8; PAYLOAD_SIZE];
	payload[0] = 2;
	payload[1..5].copy_from_slice(&offset.to_le_bytes());
	payload[5..7].copy_from_slice(&length.to_le_bytes());
	let len = data.len().min(PAYLOAD_SIZE - 7);
	payload[7..7 + len].copy_from_slice(&data[..len]);
	Packet::new(CMD_FLASH, &payload)
}

pub fn build_flash_readback() -> Result<Packet> {
	let mut payload = [0u8; PAYLOAD_SIZE];
	payload[0] = 3;
	Packet::new(CMD_FLASH, &payload)
}

pub fn build_factory_reset() -> Result<Packet> {
	Packet::new(CMD_FACTORY_RESET, &[0; PAYLOAD_SIZE])
}

pub fn parse_power_stats(packet: &Packet) -> Result<PowerStats> {
	if packet.cmd_id() != CMD_POWER {
		bail!("expected power stats, got cmd {}", packet.cmd_id());
	}
	let p = packet.payload();
	Ok(PowerStats {
		bus_voltage: u16::from_le_bytes([p[0], p[1]]),
		current_port1: u16::from_le_bytes([p[2], p[3]]),
		current_port2: u16::from_le_bytes([p[4], p[5]]),
		current_port3: u16::from_le_bytes([p[6], p[7]]),
	})
}

pub fn parse_log(packet: &Packet) -> Result<String> {
	if packet.cmd_id() != CMD_LOG {
		bail!("expected log, got cmd {}", packet.cmd_id());
	}
	let p = packet.payload();
	let len = (p[0] as usize).min(p.len() - 1);
	Ok(String::from_utf8_lossy(&p[1..1 + len]).to_string())
}
