extern crate alloc;

use core::fmt::Write;
use alloc::format;
use r_efi::efi;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

/// A Trait for a Rust-UEFI debugging library that use's the crate `log`'s macros.
pub trait DebugLib {
    fn init(image_handle: efi::Handle, system_table: *mut efi::SystemTable);
}

/// A Null implementation of the Debug Library
pub struct DebugLibNull;

impl DebugLib for DebugLibNull {
    fn init(_: efi::Handle, _: *mut efi::SystemTable) {
        // Do nothing
    }
}


// The static serial Port that DebugLibBase will write to.
lazy_static! {
  static ref SERIAL1: Mutex<SerialPort> = {
    let mut serial_port = unsafe { SerialPort::new(0x402) };
    serial_port.init();
    Mutex::new(serial_port)
  };
}

/// A Base implementation for DebugLib.
/// 
/// ## Functionality
/// 
/// This implementation writes log messages directly to the underlying serial port.
pub struct DebugLibBase;
impl log::Log for DebugLibBase {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            interrupts::without_interrupts(|| {
                let serial_lock = SERIAL1.try_lock();
                if let Some(mut serial) = serial_lock {
                    let msg = format!("{} - {}\n", record.level(), record.args());
                    serial.write_str(&msg).expect("Printing to serial failed");
                }     
            });
        }
    }

    fn flush(&self) {
        // Do nothing
    }
}

impl DebugLib for DebugLibBase {
    fn init(_: efi::Handle, _: *mut efi::SystemTable) {
        log::set_logger(&DebugLibBase)
            .map(|()| log::set_max_level(log::LevelFilter::Debug)).unwrap();
    }
}