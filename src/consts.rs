use std::time::Duration;

pub const PACKET_SIZE: usize = 256;
pub const PAYLOAD_SIZE: usize = 251;
pub const CRC_OFFSET: usize = 252;

pub const CHUNK_DATA_SIZE: usize = 240;

pub const CMD_HANDSHAKE: u8 = 1;
pub const CMD_CONFIG: u8 = 3;
pub const CMD_FACTORY_RESET: u8 = 6;
pub const CMD_FLASH: u8 = 8;
pub const CMD_POWER: u8 = 9;
pub const CMD_LOG: u8 = 10;

pub const FLASH_HEADER_AREA: usize = 8192;
pub const FRAME_HEADER_SIZE: usize = 28;
pub const MAX_FRAME_HEADERS: usize = 292;
pub const FRAME_MAGIC: u32 = 0xC019_0001;
pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 170;
pub const FRAME_PIXEL_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 2) as usize;

pub const SERIAL_BAUD_RATE: u32 = 115200;
pub const NORMAL_TIMEOUT: Duration = Duration::from_millis(2000);
pub const ERASE_TIMEOUT: Duration = Duration::from_secs(60);
