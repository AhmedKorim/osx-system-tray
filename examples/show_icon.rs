use std::thread;
use std::time::Duration;

use osx_system_tray::OsxSystemTray;

pub fn main() {
	const ICON_BUFFER: &'static [u8] = include_bytes!("rust-logo.png");
	let mut osx_tray = OsxSystemTray::new();
	osx_tray.set_tray_icon_from_buffer(ICON_BUFFER);
	osx_tray.run();
	loop {
		thread::sleep(Duration::from_millis(100));
	}
}
