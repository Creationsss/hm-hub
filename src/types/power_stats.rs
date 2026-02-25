#[derive(Debug)]
pub struct PowerStats {
	pub bus_voltage: u16,
	pub current_port1: u16,
	pub current_port2: u16,
	pub current_port3: u16,
}
