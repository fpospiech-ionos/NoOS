use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

lazy_static!{
    static ref SERIAL: Mutex<Uart16550Tty<PioBackend>> = Mutex::new(
        unsafe { Uart16550Tty::new_port(0x3f8, Config::default()).expect("[ERROR] Could not init Serial Port from 0x3f78") }
    );
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL.lock()
            .write_fmt(args).expect("[ERROR] Could not write to Serial")
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::serial::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    };
}