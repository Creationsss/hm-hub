use anyhow::{bail, Result};
use std::fmt;

#[derive(Debug, Clone)]
pub struct DeviceConfig {
	pub cur_lang: u8,
	pub web_help_onoff: u8,
	pub memory_page: u8,
	pub screen_dir: u8,
	pub screen_brightness: u8,
	pub album_cut_black: u8,
	pub album_cut_frame: u8,
	pub fun_single_click: u8,
	pub fun_double_click: u8,
	pub fun_tilt: u8,
	pub fun_shake: u8,
	pub fun_shake_sens: u8,
	pub screen_onoff_by_usb: u8,
	pub reserve1: u8,
	pub reserve2: u8,
	pub reserve3: u8,
	pub reserve4: u8,
	pub reserve5: u8,
	pub power_style: u8,
	pub image_switch_random: u8,
	pub image_switch_mode: u16,
	pub image_switch_interval: u8,
	pub srgb_style: u8,
}

impl DeviceConfig {
	pub fn from_bytes(data: &[u8]) -> Result<Self> {
		if data.len() < 24 {
			bail!("config data too short: {} < 24", data.len());
		}
		Ok(Self {
			cur_lang: data[0],
			web_help_onoff: data[1],
			memory_page: data[2],
			screen_dir: data[3],
			screen_brightness: data[4],
			album_cut_black: data[5],
			album_cut_frame: data[6],
			fun_single_click: data[7],
			fun_double_click: data[8],
			fun_tilt: data[9],
			fun_shake: data[10],
			fun_shake_sens: data[11],
			screen_onoff_by_usb: data[12],
			reserve1: data[13],
			reserve2: data[14],
			reserve3: data[15],
			reserve4: data[16],
			reserve5: data[17],
			power_style: data[18],
			image_switch_random: data[19],
			image_switch_mode: u16::from_le_bytes([data[20], data[21]]),
			image_switch_interval: data[22],
			srgb_style: data[23],
		})
	}

	pub fn to_bytes(&self) -> [u8; 24] {
		let mut b = [0u8; 24];
		b[0] = self.cur_lang;
		b[1] = self.web_help_onoff;
		b[2] = self.memory_page;
		b[3] = self.screen_dir;
		b[4] = self.screen_brightness;
		b[5] = self.album_cut_black;
		b[6] = self.album_cut_frame;
		b[7] = self.fun_single_click;
		b[8] = self.fun_double_click;
		b[9] = self.fun_tilt;
		b[10] = self.fun_shake;
		b[11] = self.fun_shake_sens;
		b[12] = self.screen_onoff_by_usb;
		b[13] = self.reserve1;
		b[14] = self.reserve2;
		b[15] = self.reserve3;
		b[16] = self.reserve4;
		b[17] = self.reserve5;
		b[18] = self.power_style;
		b[19] = self.image_switch_random;
		let mode = self.image_switch_mode.to_le_bytes();
		b[20] = mode[0];
		b[21] = mode[1];
		b[22] = self.image_switch_interval;
		b[23] = self.srgb_style;
		b
	}

	pub fn set_field(&mut self, name: &str, value: &str) -> Result<()> {
		match name {
			"brightness" | "screen_brightness" => {
				let v: u8 = value.parse()?;
				if v > 30 {
					bail!("brightness must be 0-30");
				}
				self.screen_brightness = v;
			}
			"rotation" | "screen_dir" => {
				self.screen_dir = match value {
					"0" => 0,
					"180" => 1,
					"90" => 2,
					"270" => 3,
					_ => bail!("rotation must be 0, 90, 180, or 270"),
				};
			}
			"page" | "memory_page" => {
				let v: u8 = value.parse()?;
				self.memory_page = v;
			}
			"interval" | "image_switch_interval" => {
				let v: u8 = value.parse()?;
				self.image_switch_interval = v;
			}
			"random" | "image_switch_random" => {
				let v: u8 = value.parse()?;
				self.image_switch_random = v;
			}
			"crop" | "album_cut_black" => {
				let v: u8 = value.parse()?;
				self.album_cut_black = v;
			}
			"screen_onoff_by_usb" => {
				let v: u8 = value.parse()?;
				self.screen_onoff_by_usb = v;
			}
			"shake_sens" | "fun_shake_sens" => {
				let v: u8 = value.parse()?;
				self.fun_shake_sens = v;
			}
			"power_style" => {
				let v: u8 = value.parse()?;
				self.power_style = v;
			}
			"srgb_style" => {
				let v: u8 = value.parse()?;
				self.srgb_style = v;
			}
			"switch_mode" | "image_switch_mode" => {
				let v: u16 = value.parse()?;
				self.image_switch_mode = v;
			}
			_ => bail!("unknown config field: {name}"),
		}
		Ok(())
	}
}

impl fmt::Display for DeviceConfig {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let rotation = match self.screen_dir {
			0 => "0\u{00b0}",
			1 => "180\u{00b0}",
			2 => "90\u{00b0}",
			3 => "270\u{00b0}",
			_ => "unknown",
		};
		writeln!(f, "Screen brightness:    {}/30", self.screen_brightness)?;
		writeln!(f, "Screen rotation:      {rotation}")?;
		writeln!(f, "Screen on/off by USB: {}", self.screen_onoff_by_usb != 0)?;
		writeln!(f, "Memory page:          {}", self.memory_page)?;
		writeln!(f, "Album crop to fill:   {}", self.album_cut_black != 0)?;
		writeln!(f, "Album cut frame:      {}", self.album_cut_frame != 0)?;
		writeln!(f, "Image switch random:  {}", self.image_switch_random != 0)?;
		writeln!(f, "Image switch mode:    {}", self.image_switch_mode)?;
		writeln!(
			f,
			"Image switch interval:{} sec",
			self.image_switch_interval
		)?;
		writeln!(f, "Shake sensitivity:    {}", self.fun_shake_sens)?;
		writeln!(f, "Power style:          {}", self.power_style)?;
		writeln!(f, "sRGB style:           {}", self.srgb_style)?;
		writeln!(f, "Language:             {}", self.cur_lang)?;
		write!(f, "Web help:             {}", self.web_help_onoff != 0)
	}
}
