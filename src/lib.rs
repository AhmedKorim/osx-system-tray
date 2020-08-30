#[macro_use]
extern crate objc;

use std::ops::DerefMut;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use bincode::Error as BinCodeError;
use cocoa::appkit::NSButton;
use cocoa::appkit::NSStatusItem;
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps,
    NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSImage, NSMenu, NSMenuItem,
    NSRunningApplication, NSSquareStatusItemLength, NSStatusBar, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, selector, NO};
use cocoa::foundation::{
    NSAutoreleasePool, NSData, NSPoint, NSProcessInfo, NSRect, NSSize, NSString,
};
use objc::runtime::Object;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct OsxSystemTray {
    pub app: SafeId,
    pub tray: SafeId,
    pub handler: Sender<OsxSystemTrayEvent>,
}

#[derive(Clone, Debug)]
pub struct SafeId(Arc<Mutex<*mut Object>>);

pub enum OsxSystemTrayEvent {
    ChangeImage(Vec<u8>),
    Shutdown,
}

unsafe impl Send for SafeId {}

unsafe impl Sync for SafeId {}
// unsafe impl Send for OsxSystemTray {}

impl OsxSystemTray {
    pub fn new_with_app(app: id) -> Self {
        let tray = unsafe { OsxSystemTray::init_tray() };
        let (event_tx, event_rx) = channel();
        let mut osx_tray = OsxSystemTray {
            app: SafeId(Arc::new(Mutex::new(app))),
            tray: SafeId(Arc::new(Mutex::new(tray))),
            handler: event_tx,
        };
        osx_tray.run_lister(event_rx);
        osx_tray
    }
    pub fn new() -> Self {
        let mut app;
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
        }
        let tray = unsafe { OsxSystemTray::init_tray() };
        let (event_tx, event_rx) = channel();
        let mut osx_tray = OsxSystemTray {
            app: SafeId(Arc::new(Mutex::new(app))),
            tray: SafeId(Arc::new(Mutex::new(tray))),
            handler: event_tx,
        };
        osx_tray.run_lister(event_rx);
        osx_tray
    }

    pub fn run(&mut self) {
        unsafe {
            self.app.0.clone().lock().unwrap().run();
        }
    }
    fn run_lister(&mut self, rx: Receiver<OsxSystemTrayEvent>) -> JoinHandle<()> {
        let tray = self.tray.clone();
        thread::spawn(move || loop {
            let lister = rx.try_iter();
            for e in lister {
                match e {
                    OsxSystemTrayEvent::ChangeImage(image) => unsafe {
                        const ICON_WIDTH: f64 = 32.0;
                        const ICON_HEIGHT: f64 = 32.0;
                        let nsdata = NSData::dataWithBytes_length_(
                            nil,
                            image.as_ptr() as *const std::os::raw::c_void,
                            image.len() as u64,
                        )
                        .autorelease();

                        let nsimage = unsafe {
                            NSImage::initWithData_(NSImage::alloc(nil), nsdata).autorelease()
                        };
                        let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);

                        let r: () = msg_send![nsimage, setSize: new_size];
                        tray.0.lock().unwrap().button().setImage_(nsimage);
                    },
                    OsxSystemTrayEvent::Shutdown => {
                        unimplemented!();
                    }
                }
            }
        })
    }
    fn init_tray() -> id {
        unsafe {
            NSStatusBar::systemStatusBar(nil)
                .statusItemWithLength_(NSSquareStatusItemLength)
                .autorelease()
        }
    }
    pub fn set_tray_icon_from_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        unsafe {
            const ICON_WIDTH: f64 = 18.0;
            const ICON_HEIGHT: f64 = 18.0;
            let nsdata = NSData::dataWithBytes_length_(
                nil,
                buffer.as_ptr() as *const std::os::raw::c_void,
                buffer.len() as u64,
            )
            .autorelease();

            let nsimage =
                unsafe { NSImage::initWithData_(NSImage::alloc(nil), nsdata).autorelease() };
            let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);

            let r: () = msg_send![nsimage, setSize: new_size];
            self.tray
                .0
                .clone()
                .lock()
                .unwrap()
                .button()
                .setImage_(nsimage);
        };
        Ok(())
    }
    pub fn get_emitter(&mut self) {}
}

unsafe fn set_tray_icon(tray: id, buffer: &[u8]) {
    const ICON_WIDTH: f64 = 32.0;
    const ICON_HEIGHT: f64 = 32.0;
    let nsdata = NSData::dataWithBytes_length_(
        nil,
        buffer.as_ptr() as *const std::os::raw::c_void,
        buffer.len() as u64,
    )
    .autorelease();

    let nsimage = unsafe { NSImage::initWithData_(NSImage::alloc(nil), nsdata).autorelease() };
    let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);
    let r: () = msg_send![nsimage, setSize: new_size];
    tray.button().setImage_(nsimage);
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    BinCodeError(#[from] BinCodeError),
    #[error("File doesn't contain meta data")]
    NoMetaData,
    #[error("File doesn't contain meta data")]
    PoisonError,
}

#[cfg(test)]
mod tests {
    use crate::OsxSystemTray;

    #[test]
    fn it_works() {
        const ICON_BUFFER: &'static [u8] = include_bytes!("rust-logo.png");
        let mut osx_tray = OsxSystemTray::new();
        osx_tray.set_tray_icon_from_buffer(ICON_BUFFER);
    }
}
