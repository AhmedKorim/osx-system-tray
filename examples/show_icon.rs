use std::thread;
use std::time::Duration;

use osx_system_tray::{OsxSystemTray,OsxSystemTrayEvent};

pub fn main() {
	const ICON_BUFFER: &'static [u8] = include_bytes!("rust-logo.png");
	let mut osx_tray = OsxSystemTray::new();
	osx_tray.set_tray_icon_from_buffer(ICON_BUFFER);
	let rx = osx_tray.handler.clone();
	let join = thread::spawn(move || {
		loop {
			dbg!("looping over");
			rx.send(OsxSystemTrayEvent::ChangeImage(ICON_BUFFER));
			thread::sleep(Duration::from_millis(1000));
		}
	});
	osx_tray.run();
}
