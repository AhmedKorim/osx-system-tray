use std::thread;
use std::time::Duration;

use image::png::{FilterType, PNGEncoder};
use image::{
    imageops::rotate90, load_from_memory, load_from_memory_with_format, png::CompressionType,
    DynamicImage, GenericImageView, ImageFormat,
};

use osx_system_tray::{OsxSystemTray, OsxSystemTrayEvent};

pub fn main() {
    const ICON_BUFFER: &'static [u8] = include_bytes!("480px-Servo_logo.png");
    let mut osx_tray = OsxSystemTray::new();
    osx_tray.set_tray_icon_from_buffer(ICON_BUFFER);
    let rx = osx_tray.handler.clone();
    let join = thread::spawn(move || {
        let mut dn_image = load_from_memory_with_format(ICON_BUFFER, ImageFormat::Png).unwrap();
        let dn_image_rotate90 = dn_image.rotate90();
        let dn_image_rotate180 = dn_image.rotate180();
        let dn_image_rotate270 = dn_image.rotate270();

        let mut deg = 0;
        loop {
            let mut image = dn_image.clone();
            if deg == 0 || deg == 360 {
            } else if deg == 90 {
                image = dn_image_rotate90.clone();
            } else if deg == 180 {
                image = dn_image_rotate180.clone();
            } else if deg == 270 {
                image = dn_image_rotate270.clone();
            }
            let image_buf = get_image_bytes(&image);
            rx.send(OsxSystemTrayEvent::ChangeImage(image_buf)).unwrap();
            thread::sleep(Duration::from_millis(10));
            deg += 90;
            if deg == 360 {
                deg = 0;
            }
        }
    });
    osx_tray.run();
}

fn get_image_bytes(image: &DynamicImage) -> Vec<u8> {
    let mut output = Vec::new();
    let mut j =
        PNGEncoder::new_with_quality(&mut output, CompressionType::Fast, FilterType::NoFilter);
    j.encode(
        &image.to_bytes(),
        image.width(),
        image.height(),
        image.color(),
    )
    .unwrap();
    output
}
