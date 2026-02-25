mod cli;
mod consts;
mod device;
mod flash;
mod image;
mod protocol;
mod types;

use anyhow::{bail, Result};
use clap::Parser;
use std::path::Path;

use cli::{Cli, Commands, ConfigAction};
use consts::*;
use device::Device;
use types::FrameHeader;

fn main() -> Result<()> {
	let cli = Cli::parse();

	let port = match &cli.port {
		Some(p) => p.clone(),
		None => {
			if matches!(cli.command, Commands::Config { action: Some(ConfigAction::Set { ref field, .. }) } if field.is_none())
			{
				String::new()
			} else {
				device::detect_port()?
			}
		}
	};

	match cli.command {
		Commands::Info => cmd_info(&port),
		Commands::Config { action } => cmd_config(&port, action),
		Commands::Upload {
			images,
			no_crop,
			preview,
		} => cmd_upload(&port, &images, !no_crop, preview.as_deref()),
		Commands::Slideshow { dir, no_crop } => cmd_slideshow(&port, &dir, !no_crop),
		Commands::Power { watch } => cmd_power(&port, watch),
		Commands::Monitor => cmd_monitor(&port),
		Commands::Read { output } => cmd_read(&port, &output),
		Commands::Reset => cmd_reset(&port),
		Commands::Backup { file } => cmd_backup(&port, &file),
		Commands::Restore { file } => cmd_restore(&port, &file),
		Commands::Rotate {
			dir,
			interval,
			no_crop,
		} => cmd_rotate(&port, &dir, interval, !no_crop),
	}
}

fn cmd_info(port: &str) -> Result<()> {
	let dev = Device::open(port)?;
	let info = &dev.info;
	println!("HM Lab Z-NEO 8K USB Hub");
	println!("  Hardware ID:    {:#010x}", info.hw_id);
	println!("  Firmware:       {}", info.fw_version_string());
	println!("  Flash size:     {} MB", info.flash_size / 1024 / 1024);
	println!("  Max frames:     {}", info.max_frames());
	Ok(())
}

fn cmd_config(port: &str, action: Option<ConfigAction>) -> Result<()> {
	match action {
		None => {
			let mut dev = Device::open(port)?;
			let config = dev.read_config()?;
			println!("{config}");
		}
		Some(ConfigAction::Set { field, value }) => match (field, value) {
			(Some(f), Some(v)) => {
				let mut dev = Device::open(port)?;
				let mut config = dev.read_config()?;
				config.set_field(&f, &v)?;
				dev.write_config(&config)?;
				println!("Set {f} = {v}");
			}
			_ => {
				println!("Available config fields:");
				println!("  brightness <0-30>        Screen brightness");
				println!("  rotation <0|90|180|270>  Screen rotation");
				println!("  interval <seconds>       Image switch interval");
				println!("  random <0|1>             Random image order");
				println!("  crop <0|1>               Crop to fill (1) or letterbox (0)");
				println!("  shake_sens <0-255>       Shake sensitivity");
				println!("  screen_onoff_by_usb <0|1> Screen on/off with USB");
				println!("  power_style <0-255>      Power display style");
				println!("  srgb_style <0-255>       sRGB style");
				println!("  switch_mode <0-65535>    Image switch mode");
				println!("  page <0-255>             Memory page");
			}
		},
		Some(ConfigAction::Dump) => {
			let mut dev = Device::open(port)?;
			let config = dev.read_config()?;
			let bytes = config.to_bytes();
			for (i, b) in bytes.iter().enumerate() {
				if i > 0 && i % 16 == 0 {
					println!();
				}
				print!("{b:02x} ");
			}
			println!();
		}
	}
	Ok(())
}

fn cmd_upload(
	port: &str,
	images: &[std::path::PathBuf],
	crop: bool,
	preview: Option<&Path>,
) -> Result<()> {
	let mut albums = Vec::new();
	for path in images {
		eprintln!("Loading {}...", path.display());
		let album = crate::image::load_image(path, crop)?;
		eprintln!(
			"  {} frame(s), {}x{}",
			album.frames.len(),
			DISPLAY_WIDTH,
			DISPLAY_HEIGHT
		);
		albums.push(album);
	}

	if let Some(preview_path) = preview {
		if let Some(first_frame) = albums.first().and_then(|a| a.frames.first()) {
			let img = rgb565_to_image(first_frame, DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16);
			img.save(preview_path)?;
			println!("Preview saved to {}", preview_path.display());
		}
		return Ok(());
	}

	let mut dev = Device::open(port)?;
	let max = dev.info.max_frames();
	let total_frames: usize = albums.iter().map(|a| a.frames.len()).sum();
	eprintln!("Total: {total_frames} frame(s) (max: {max})");

	let flash_data = flash::build_flash_buffer(&albums, dev.info.flash_size)?;
	dev.upload_flash(&flash_data)?;
	Ok(())
}

fn cmd_slideshow(port: &str, dir: &Path, crop: bool) -> Result<()> {
	if !dir.is_dir() {
		bail!("{} is not a directory", dir.display());
	}

	let paths = collect_images(dir)?;
	if paths.is_empty() {
		bail!("no images found in {}", dir.display());
	}

	eprintln!("Found {} image(s) in {}", paths.len(), dir.display());

	let mut albums = Vec::new();
	for path in &paths {
		eprintln!("Loading {}...", path.display());
		let album = crate::image::load_image(path, crop)?;
		eprintln!("  {} frame(s)", album.frames.len());
		albums.push(album);
	}

	let mut dev = Device::open(port)?;
	let max = dev.info.max_frames();
	let total_frames: usize = albums.iter().map(|a| a.frames.len()).sum();
	eprintln!("Total: {total_frames} frame(s) (max: {max})");

	let flash_data = flash::build_flash_buffer(&albums, dev.info.flash_size)?;
	dev.upload_flash(&flash_data)?;
	Ok(())
}

fn cmd_power(port: &str, watch: bool) -> Result<()> {
	let mut dev = Device::open(port)?;

	loop {
		let stats = dev.read_power()?;
		let voltage = stats.bus_voltage as f64 / 1000.0;
		let rating = if stats.bus_voltage >= 4750 {
			"Healthy"
		} else if stats.bus_voltage >= 4250 {
			"Warning"
		} else {
			"Critical"
		};

		if watch {
			eprint!(
				"\rBus: {voltage:.2}V ({rating}) | Ports: {}mA {}mA {}mA   ",
				stats.current_port1, stats.current_port2, stats.current_port3
			);
		} else {
			println!("Bus voltage:  {voltage:.2}V ({rating})");
			println!("Port 1:       {}mA", stats.current_port1);
			println!("Port 2:       {}mA", stats.current_port2);
			println!("Port 3:       {}mA", stats.current_port3);
			return Ok(());
		}
	}
}

fn cmd_monitor(port: &str) -> Result<()> {
	let mut dev = Device::open(port)?;
	eprintln!("Monitoring device (Ctrl+C to stop)...");
	dev.monitor()
}

fn cmd_read(port: &str, output: &Path) -> Result<()> {
	let mut dev = Device::open(port)?;
	let flash_data = dev.read_flash()?;

	std::fs::create_dir_all(output)?;

	let mut i = 0;
	loop {
		let offset = i * FRAME_HEADER_SIZE;
		if offset + FRAME_HEADER_SIZE > FLASH_HEADER_AREA {
			break;
		}
		let header = match FrameHeader::read_from(&flash_data[offset..])? {
			Some(h) => h,
			None => break,
		};

		let start = header.data_offset as usize;
		let end = start + header.data_length as usize;
		if end > flash_data.len() {
			break;
		}

		let pixel_data = &flash_data[start..end];

		if header.frame_count == 1 {
			let img = rgb565_to_image(pixel_data, header.width, header.height);
			let out_path = output.join(format!("frame_{i}.png"));
			img.save(&out_path)?;
			println!("Saved {}", out_path.display());
		} else {
			let frame_size = (header.width as usize) * (header.height as usize) * 2;
			for f in 0..header.frame_count as usize {
				let fstart = f * frame_size;
				let fend = fstart + frame_size;
				if fend > pixel_data.len() {
					break;
				}
				let img = rgb565_to_image(&pixel_data[fstart..fend], header.width, header.height);
				let out_path = output.join(format!("frame_{i}_{f}.png"));
				img.save(&out_path)?;
				println!("Saved {}", out_path.display());
			}
		}

		i += 1;
	}

	if i == 0 {
		println!("No images found on device.");
	}
	Ok(())
}

fn cmd_reset(port: &str) -> Result<()> {
	let mut dev = Device::open(port)?;
	dev.factory_reset()?;
	println!("Factory reset sent.");
	Ok(())
}

fn cmd_backup(port: &str, file: &Path) -> Result<()> {
	let mut dev = Device::open(port)?;

	eprintln!("Reading config...");
	let config = dev.read_config()?;
	let config_bytes = config.to_bytes();

	eprintln!("Reading flash...");
	let flash_data = dev.read_flash()?;

	let mut backup = Vec::new();
	backup.extend_from_slice(b"HMHUB\x01");
	backup.extend_from_slice(&(config_bytes.len() as u32).to_le_bytes());
	backup.extend_from_slice(&config_bytes);
	backup.extend_from_slice(&(flash_data.len() as u32).to_le_bytes());
	backup.extend_from_slice(&flash_data);
	let checksum = crc32fast::hash(&backup);
	backup.extend_from_slice(&checksum.to_le_bytes());

	std::fs::write(file, &backup)?;
	println!(
		"Backup saved to {} ({:.1} MB)",
		file.display(),
		backup.len() as f64 / 1_048_576.0
	);
	Ok(())
}

fn cmd_restore(port: &str, file: &Path) -> Result<()> {
	let data = std::fs::read(file)?;

	if data.len() < 14 || &data[..5] != b"HMHUB" {
		bail!("not a valid hm-hub backup file");
	}
	if data[5] != 1 {
		bail!("unsupported backup version: {}", data[5]);
	}

	let stored_crc = u32::from_le_bytes([
		data[data.len() - 4],
		data[data.len() - 3],
		data[data.len() - 2],
		data[data.len() - 1],
	]);
	let computed_crc = crc32fast::hash(&data[..data.len() - 4]);
	if stored_crc != computed_crc {
		bail!("backup file is corrupted (CRC mismatch)");
	}

	let mut pos = 6;

	let config_len =
		u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
	pos += 4;
	let config = types::DeviceConfig::from_bytes(&data[pos..pos + config_len])?;
	pos += config_len;

	let flash_len =
		u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
	pos += 4;
	let flash_data = &data[pos..pos + flash_len];

	let mut dev = Device::open(port)?;

	eprintln!("Restoring config...");
	dev.write_config(&config)?;

	eprintln!("Restoring flash...");
	dev.upload_flash(flash_data)?;

	println!("Restore complete.");
	Ok(())
}

fn collect_images(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
	let mut paths: Vec<_> = std::fs::read_dir(dir)?
		.filter_map(|e| e.ok())
		.map(|e| e.path())
		.filter(|p| {
			p.extension()
				.and_then(|e| e.to_str())
				.map(|e| {
					matches!(
						e.to_lowercase().as_str(),
						"png" | "jpg" | "jpeg" | "bmp" | "webp" | "gif"
					)
				})
				.unwrap_or(false)
		})
		.collect();
	paths.sort();
	Ok(paths)
}

fn dir_fingerprint(dir: &Path) -> Result<u32> {
	let mut hasher = crc32fast::Hasher::new();
	let paths = collect_images(dir)?;
	for path in &paths {
		hasher.update(path.to_string_lossy().as_bytes());
		let meta = std::fs::metadata(path)?;
		let modified = meta
			.modified()?
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default();
		hasher.update(&modified.as_secs().to_le_bytes());
		hasher.update(&meta.len().to_le_bytes());
	}
	Ok(hasher.finalize())
}

fn cmd_rotate(port: &str, dir: &Path, interval: u64, crop: bool) -> Result<()> {
	if !dir.is_dir() {
		bail!("{} is not a directory", dir.display());
	}

	eprintln!(
		"Watching {} for changes every {}s (Ctrl+C to stop)...",
		dir.display(),
		interval
	);

	let mut last_fingerprint: u32 = 0;

	loop {
		let fingerprint = dir_fingerprint(dir)?;
		if fingerprint != last_fingerprint {
			let paths = collect_images(dir)?;
			if paths.is_empty() {
				eprintln!("No images found, waiting...");
			} else {
				eprintln!("Change detected, uploading {} image(s)...", paths.len());
				let mut albums = Vec::new();
				for path in &paths {
					let album = crate::image::load_image(path, crop)?;
					albums.push(album);
				}

				let mut dev = Device::open(port)?;
				let flash_data = flash::build_flash_buffer(&albums, dev.info.flash_size)?;
				dev.upload_flash(&flash_data)?;
				eprintln!("Upload complete, watching for changes...");
			}
			last_fingerprint = fingerprint;
		}
		std::thread::sleep(std::time::Duration::from_secs(interval));
	}
}

fn rgb565_to_image(data: &[u8], width: u16, height: u16) -> ::image::RgbaImage {
	let w = width as u32;
	let h = height as u32;
	let mut img = ::image::RgbaImage::new(w, h);

	for y in 0..h {
		for x in 0..w {
			let idx = ((y * w + x) * 2) as usize;
			if idx + 1 >= data.len() {
				break;
			}
			let hi = data[idx] as u16;
			let lo = data[idx + 1] as u16;
			let pixel = (hi << 8) | lo;

			let r = ((pixel >> 11) & 0x1F) as u8;
			let g = ((pixel >> 5) & 0x3F) as u8;
			let b = (pixel & 0x1F) as u8;

			let r8 = (r << 3) | (r >> 2);
			let g8 = (g << 2) | (g >> 4);
			let b8 = (b << 3) | (b >> 2);

			img.put_pixel(x, y, ::image::Rgba([r8, g8, b8, 255]));
		}
	}
	img
}
