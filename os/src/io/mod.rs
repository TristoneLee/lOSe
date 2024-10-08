pub mod uart;

use core::fmt;
use core::fmt::Write;
use crate::io::uart::{uart_init, uart_putchar, uart_work};

struct STDOUT;

pub fn init() {
    unsafe {
        uart_init();
    }
}

impl Write for STDOUT {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe {
                uart_putchar(c as u8);
                uart_work();
            }
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        unsafe {
            uart_putchar(c as u8);
            uart_work();
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    STDOUT.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


