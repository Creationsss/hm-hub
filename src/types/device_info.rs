use crate::consts::{FLASH_HEADER_AREA, FRAME_PIXEL_SIZE};

#[derive(Debug)]
pub struct DeviceInfo {
	pub hw_id: u32,
	pub fw_ver: u32,
	pub flash_size: u32,
}

impl DeviceInfo {
	pub fn max_frames(&self) -> usize {
		max_frames(self.flash_size)
	}

	pub fn fw_version_string(&self) -> String {
		let major = (self.fw_ver >> 16) & 0xFF;
		let minor = (self.fw_ver >> 8) & 0xFF;
		let patch = self.fw_ver & 0xFF;
		format!("{major}.{minor}.{patch}")
	}
}

pub fn max_frames(flash_size: u32) -> usize {
	(flash_size as usize).saturating_sub(FLASH_HEADER_AREA) / FRAME_PIXEL_SIZE
}
