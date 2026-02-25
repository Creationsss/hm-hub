use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hm-hub", about = "CLI for HM Lab Z-NEO 8K USB Hub")]
pub struct Cli {
	#[arg(short, long, help = "Serial port path (auto-detects if not specified)")]
	pub port: Option<String>,

	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	#[command(about = "Show device info (HW ID, firmware, flash size)")]
	Info,
	#[command(about = "Read or set device config")]
	Config {
		#[command(subcommand)]
		action: Option<ConfigAction>,
	},
	#[command(about = "Upload images/GIFs to the device LCD")]
	Upload {
		#[arg(required = true)]
		images: Vec<PathBuf>,

		#[arg(long, help = "Letterbox instead of cropping to fill")]
		no_crop: bool,

		#[arg(long, help = "Save a preview PNG instead of uploading")]
		preview: Option<PathBuf>,
	},
	#[command(about = "Upload all images from a directory")]
	Slideshow {
		#[arg(help = "Directory containing images")]
		dir: PathBuf,

		#[arg(long, help = "Letterbox instead of cropping to fill")]
		no_crop: bool,
	},
	#[command(about = "Show USB power/current stats")]
	Power {
		#[arg(short, long, help = "Continuously monitor power stats")]
		watch: bool,
	},
	#[command(about = "Live device log and power monitor")]
	Monitor,
	#[command(about = "Read back stored images from device flash")]
	Read {
		#[arg(
			short,
			long,
			default_value = ".",
			help = "Output directory for saved images"
		)]
		output: PathBuf,
	},
	#[command(about = "Factory reset the device")]
	Reset,
	#[command(about = "Backup device config and flash to a file")]
	Backup {
		#[arg(help = "Output file path")]
		file: PathBuf,
	},
	#[command(about = "Restore device config and flash from a backup")]
	Restore {
		#[arg(help = "Backup file path")]
		file: PathBuf,
	},
	#[command(about = "Watch a directory and re-upload when images change")]
	Rotate {
		#[arg(help = "Directory containing images")]
		dir: PathBuf,

		#[arg(
			long,
			default_value_t = 60,
			help = "Seconds between checks for changes"
		)]
		interval: u64,

		#[arg(long, help = "Letterbox instead of cropping to fill")]
		no_crop: bool,
	},
}

#[derive(Subcommand)]
pub enum ConfigAction {
	#[command(about = "Set a config field (e.g. brightness 20, rotation 90)")]
	Set {
		field: Option<String>,
		value: Option<String>,
	},
	#[command(about = "Dump raw config bytes (hex)")]
	Dump,
}
