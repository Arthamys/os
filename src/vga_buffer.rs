use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(test)]
use crate::{serial_print, serial_println};

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Ensures that the struct maps to u8 and only that (no fuss possible)
// this allows easy transumtation between the struct and the underlying
// data.
// Note that this is only available for unit structs.
// see: https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// We use repr(C) to force the ordering of the struct's fields.
// Otherwise, Rust does not guarantee the same ordering as was defined.
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Same, we wans to make sure Buffer has the same layout as it's wrapped data
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode, // color code in use for printing
    // This has an explicit 'static lifetime because it references the VGA
    // buffer. As this is memory mapped hardware, it will be valid for the
    // whole duration of the program.
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Here we filter out values that are outside of the
                // page 437 value set. (Or non-printable ASCII (0x0..0x20)
                // This is necessary because rust strings are UTF-8, which
                // represent a richer character set, not handled by VGA.
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar{
                    ascii_char: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            let blank = ScreenChar {
                ascii_char: b' ',
                color_code: self.color_code,
            };
            self.buffer.chars[row][col].write(blank);
        }
    }

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut w = WRITER.lock();
        writeln!(w, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = w.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });

    serial_println!("[ok]");
}
