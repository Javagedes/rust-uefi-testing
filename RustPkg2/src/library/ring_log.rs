use pkg1::interface::DebugLib;
use r_efi::efi;
use alloc::format;
use log;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;
use core::fmt::Write;

lazy_static! {
    static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x402) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

static LOGGER: RingBufferDebugLib = RingBufferDebugLib::new();
struct RingBuffer<const N: usize> {
    buffer: [u8; N],
    read_index: usize,
    write_index: usize,
}

impl<const N: usize> RingBuffer<N> {
    const fn new() -> Self {
        RingBuffer {
            buffer: [0; N],
            read_index: 0,
            write_index: 0,
        }
    }

    fn write(&mut self, data: u8) {
        self.buffer[self.write_index] = data;
        self.write_index = (self.write_index + 1) % N;
    }

    fn read(&mut self) -> u8 {
        let data = self.buffer[self.read_index];
        self.read_index = (self.read_index + 1) % N;
        data
    }
}


pub struct RingBufferDebugLib {
    buffer: Mutex<RingBuffer<1024>>,
}


impl RingBufferDebugLib {
    const fn new() -> Self {
        RingBufferDebugLib {
            buffer: Mutex::new(RingBuffer::new()),
        }
    }
}

impl DebugLib for RingBufferDebugLib {
    fn init(_: efi::Handle, _: *mut efi::SystemTable) {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Debug)).unwrap();
    }
}

impl log::Log for RingBufferDebugLib {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {

        if self.enabled(record.metadata()) {
            let msg = format!("{} - {}\n", record.level(), record.args());
            if let Some(mut buffer) = self.buffer.try_lock() {
                for byte in msg.bytes() {
                    buffer.write(byte);
                } 
            }
            self.flush();
        }
    }

    fn flush(&self) {
        interrupts::without_interrupts(|| {
            let serial_lock = SERIAL1.try_lock();
            if let Some(mut serial) = serial_lock {
                serial.write_str("Flushing: ").expect("Printing to serial failed");
                if let Some(mut buffer) = self.buffer.try_lock() {
                    while buffer.read_index != buffer.write_index {
                        let data = buffer.read();
                        serial.write_char(data as char).expect("Printing to serial failed");
                    }
                }
            }
        });
    }
}
