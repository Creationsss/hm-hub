use anyhow::{bail, Result};
use std::time::Duration;

use crate::consts::PACKET_SIZE;
use crate::types::Packet;

pub fn recv_packet(port: &mut dyn serialport::SerialPort, timeout: Duration) -> Result<Packet> {
	port.set_timeout(timeout)?;
	let mut buf = [0u8; PACKET_SIZE];
	let mut pos = 0;
	let deadline = std::time::Instant::now() + timeout;
	while pos < PACKET_SIZE {
		if std::time::Instant::now() > deadline {
			bail!("timeout waiting for packet ({pos}/{PACKET_SIZE} bytes received)");
		}
		match port.read(&mut buf[pos..]) {
			Ok(0) => bail!("serial port EOF"),
			Ok(n) => pos += n,
			Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
				if pos == 0 {
					bail!("timeout waiting for packet");
				}
			}
			Err(e) => return Err(e.into()),
		}
	}
	Packet::from_bytes(buf)
}
