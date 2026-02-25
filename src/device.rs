use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::consts::*;
use crate::protocol::chunked::encode_chunked;
use crate::protocol::commands::*;
use crate::protocol::packet::recv_packet;
use crate::types::{ChunkedReceiver, DeviceConfig, DeviceInfo};

const HM_VID: u16 = 0xC019;
const HM_PID: u16 = 0x0401;

pub struct Device {
	port: Box<dyn serialport::SerialPort>,
	pub info: DeviceInfo,
}

pub fn detect_port() -> Result<String> {
	let ports = serialport::available_ports()?;
	for p in &ports {
		if let serialport::SerialPortType::UsbPort(usb) = &p.port_type {
			if usb.vid == HM_VID && usb.pid == HM_PID {
				return Ok(p.port_name.clone());
			}
		}
	}
	bail!("no HM Lab device found (VID:{HM_VID:#06x} PID:{HM_PID:#06x}). Is it plugged in?")
}

impl Device {
	pub fn open(path: &str) -> Result<Self> {
		let port = serialport::new(path, SERIAL_BAUD_RATE)
			.data_bits(serialport::DataBits::Eight)
			.stop_bits(serialport::StopBits::One)
			.parity(serialport::Parity::None)
			.timeout(NORMAL_TIMEOUT)
			.open()?;

		let mut dev = Device {
			port,
			info: DeviceInfo {
				hw_id: 0,
				fw_ver: 0,
				flash_size: 0,
			},
		};
		dev.handshake()?;
		Ok(dev)
	}

	fn handshake(&mut self) -> Result<()> {
		let pkt = build_handshake()?;
		pkt.send(&mut *self.port)?;
		let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
		self.info = parse_handshake(&resp)?;
		Ok(())
	}

	pub fn read_config(&mut self) -> Result<DeviceConfig> {
		let pkt = build_config_read()?;
		pkt.send(&mut *self.port)?;

		let mut receiver = ChunkedReceiver::new();
		let mut retries = 0;

		loop {
			let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
			match resp.cmd_id() {
				CMD_CONFIG => {
					let payload = resp.payload();
					match payload[0] {
						1 => continue,
						2 => match receiver.feed(&payload[1..])? {
							Some(data) => return DeviceConfig::from_bytes(&data),
							None => continue,
						},
						other => {
							retries += 1;
							if retries > 10 {
								bail!("unexpected config sub-command: {other}");
							}
						}
					}
				}
				CMD_LOG => {
					if let Ok(msg) = parse_log(&resp) {
						eprintln!("[device log] {msg}");
					}
				}
				CMD_POWER => {}
				_ => {
					retries += 1;
					if retries > 10 {
						bail!("failed to read config after {retries} unexpected packets");
					}
				}
			}
		}
	}

	pub fn write_config(&mut self, config: &DeviceConfig) -> Result<()> {
		let data = config.to_bytes();
		let packets = encode_chunked(CMD_CONFIG, 2, &data)?;
		for pkt in &packets {
			pkt.send(&mut *self.port)?;
		}
		Ok(())
	}

	pub fn upload_flash(&mut self, flash_data: &[u8]) -> Result<()> {
		let total_size = flash_data.len() as u32;

		let pkt = build_flash_start(total_size)?;
		pkt.send(&mut *self.port)?;

		let spinner = ProgressBar::new_spinner();
		spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
		spinner.set_message("Waiting for flash erase...");

		loop {
			let resp = recv_packet(&mut *self.port, ERASE_TIMEOUT)?;
			if resp.cmd_id() == CMD_FLASH {
				let payload = resp.payload();
				if payload[0] == 1 {
					match payload[1] {
						2 => spinner.set_message("Erasing flash..."),
						4 => {
							spinner.finish_with_message("Erase complete.");
							break;
						}
						_ => {}
					}
				}
			} else if resp.cmd_id() == CMD_LOG {
				if let Ok(msg) = parse_log(&resp) {
					spinner.println(format!("[device log] {msg}"));
				}
			}
		}

		let pb = ProgressBar::new(flash_data.len() as u64);
		pb.set_style(
			ProgressStyle::default_bar()
				.template("{spinner:.cyan} [{bar:40.cyan/dim}] {bytes}/{total_bytes} ({eta})")?
				.progress_chars("=> "),
		);

		loop {
			let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
			if resp.cmd_id() == CMD_FLASH {
				let payload = resp.payload();
				match payload[0] {
					2 => {
						let offset =
							u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]);
						let length = u16::from_le_bytes([payload[5], payload[6]]);

						let start = offset as usize;
						let end = (start + length as usize).min(flash_data.len());
						let chunk = &flash_data[start..end];

						let resp_pkt = build_flash_data_response(offset, length, chunk)?;
						resp_pkt.send(&mut *self.port)?;

						pb.set_position(
							(offset as u64 + length as u64).min(flash_data.len() as u64),
						);
					}
					4 => {
						pb.finish_with_message("Upload complete!");
						return Ok(());
					}
					_ => {}
				}
			} else if resp.cmd_id() == CMD_LOG {
				if let Ok(msg) = parse_log(&resp) {
					pb.println(format!("[device log] {msg}"));
				}
			}
		}
	}

	pub fn read_flash(&mut self) -> Result<Vec<u8>> {
		let flash_size = self.info.flash_size as usize;

		let pkt = build_flash_readback()?;
		pkt.send(&mut *self.port)?;

		let mut buffer = vec![0u8; flash_size];

		let pb = ProgressBar::new(flash_size as u64);
		pb.set_style(
			ProgressStyle::default_bar()
				.template("{spinner:.cyan} [{bar:40.cyan/dim}] {bytes}/{total_bytes} ({eta})")?
				.progress_chars("=> "),
		);

		loop {
			let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
			if resp.cmd_id() == CMD_FLASH {
				let payload = resp.payload();
				match payload[0] {
					3 => {
						let offset =
							u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]])
								as usize;
						let length = u16::from_le_bytes([payload[5], payload[6]]) as usize;
						let data = &payload[7..7 + length];

						if offset + length <= buffer.len() {
							buffer[offset..offset + length].copy_from_slice(data);
						}

						pb.set_position((offset + length) as u64);
					}
					4 => {
						pb.finish_with_message("Read complete!");
						return Ok(buffer);
					}
					_ => {}
				}
			} else if resp.cmd_id() == CMD_LOG {
				if let Ok(msg) = parse_log(&resp) {
					pb.println(format!("[device log] {msg}"));
				}
			}
		}
	}

	pub fn read_power(&mut self) -> Result<crate::types::PowerStats> {
		loop {
			let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
			if resp.cmd_id() == CMD_POWER {
				return parse_power_stats(&resp);
			} else if resp.cmd_id() == CMD_LOG {
				if let Ok(msg) = parse_log(&resp) {
					eprintln!("[device log] {msg}");
				}
			}
		}
	}

	pub fn monitor(&mut self) -> Result<()> {
		loop {
			let resp = recv_packet(&mut *self.port, NORMAL_TIMEOUT)?;
			match resp.cmd_id() {
				CMD_POWER => {
					let stats = parse_power_stats(&resp)?;
					let voltage = stats.bus_voltage as f64 / 1000.0;
					let rating = if stats.bus_voltage >= 4750 {
						"Healthy"
					} else if stats.bus_voltage >= 4250 {
						"Warning"
					} else {
						"Critical"
					};
					eprint!(
						"\rBus: {voltage:.2}V ({rating}) | Ports: {}mA {}mA {}mA   ",
						stats.current_port1, stats.current_port2, stats.current_port3
					);
				}
				CMD_LOG => {
					if let Ok(msg) = parse_log(&resp) {
						eprintln!("\r[device log] {msg}                              ");
					}
				}
				_ => {}
			}
		}
	}

	pub fn factory_reset(&mut self) -> Result<()> {
		let pkt = build_factory_reset()?;
		pkt.send(&mut *self.port)?;
		Ok(())
	}
}
