use core::{
    arch::asm,
    fmt::{self, Write},
};

const COM1_PORT: u16 = 0x3f8;

pub fn inti_serial() {
    unsafe {
        // Disable all interrupts
        outb(COM1_PORT + 1, 0x00);
        // Enable DLAB (Divisor Latch Access Bit)
        // Allows to set the baud rate divisor
        outb(COM1_PORT + 3, 0x80);

        // Set baud rate to 38400 (Divisor = 3)
        // 115200 / 3 = 38400
        outb(COM1_PORT + 0, 0x03); // Low byte
        outb(COM1_PORT + 1, 0x00); // High byte

        // Clear DLAB and set data format: 8bits, No parity, 1 stop bit
        outb(COM1_PORT + 3, 0x03);

        // Enable FIFO, Clear, 14 byte threshold
        outb(COM1_PORT + 2, 0xC7);

        // Set Data Terminal Ready (DTR) and Request To Send (RTS)
        outb(COM1_PORT + 4, 0x0B);
    }
}

/// Reads a byte from the specified I/O port
#[inline]
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
        );
    }
}

/// Reads a byte from the specified I/O port
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let mut val: u8 = 0;

    unsafe {
        asm!("in al, dx",
        in("dx") port,
        out("al") val,
        options(nomem, nostack, preserves_flags)
        );
    }

    val
}

fn is_transmit_empty() -> bool {
    unsafe { inb(COM1_PORT + 5) & 0x20 != 0 }
}

fn write_serial_byte(byte: u8) {
    while !is_transmit_empty() {
        core::hint::spin_loop();
    }

    unsafe {
        outb(COM1_PORT, byte);
    }
}

struct SerialPort;

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            write_serial_byte(byte);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    let mut serial = SerialPort;
    serial.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        $crate::serial::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => (
        $crate::serial_print!("{}\n", core::format_args!($($arg)*));
    );
}
