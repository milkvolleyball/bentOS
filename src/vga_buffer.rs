#[allow(dead_code)]
use core::fmt::Write;
use core::fmt;

use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(test)]
use crate::{serial_print,serial_println};

lazy_static! {
    pub static ref WRITER:Mutex<Writer> = Mutex::new(Writer {
        column_position:0,
        color_code:ColorCode::new(Color::LightCyan,Color::Blue),
        buffer:unsafe { &mut *(0xb8000 as *mut Buffer)},
    });
}


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(u8)]
pub enum Color {
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

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(transparent)]//this guarantted ColorCode stay same layout with u8 in memory
struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode{
        ColorCode ((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code:ColorCode,
}

const BUFFER_HEIGHT:usize = 25;
const BUFFER_WIDTH:usize = 80;

#[repr(transparent)]
struct Buffer {
    chars:[[Volatile<ScreenChar>;BUFFER_WIDTH]; BUFFER_HEIGHT],
}


#[doc(hidden)]
pub fn _print(args:fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {$crate::vga_buffer::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ()=>($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n",format_args!($($arg)*)));
}

//Write to screen
pub struct Writer {
    column_position:usize,//cursor location in last row
    color_code:ColorCode,
    buffer: &'static mut Buffer,
}
impl Writer {
    pub fn write_byte(&mut self, byte:u8) {//byte is ASCii code
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                
                let row = BUFFER_HEIGHT -1;//25 -1 =24
                let col = self.column_position;

                let color_code = self.color_code;
                //cursor location:
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character:byte,
                    color_code,
                });
                self.column_position = self.column_position + 1;
            }
        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {  //MOVE every single character 1 up
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT-1); //clear bottom line.
        self.column_position = 0;
    }
    fn clear_row(&mut self, row:usize) {
        let blank = ScreenChar {
            ascii_character:b' ',
            color_code:self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s:&str) {
        for byte in s.bytes() {
            match byte {
                //printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),//'|' means OR
                //illegal
                _=>self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self,s:&str)->fmt::Result {
        self.write_string(s);
        Ok(())
    }
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

    serial_print!("test_println_output...");

    let s ="ABCD";
    interrupts::without_interrupts(||{
        let mut writer = WRITER.lock();
        writeln!(writer,"\n{}",s).expect("writeln failed");////print "ABCD" at [BUFFER_HEIGHT-2][0/1/2/3]
        for(i,c) in s.chars().enumerate(){//(0,A),(1,B),(2,C),(3,D)
            let scr_char = writer.buffer.chars[BUFFER_HEIGHT-2][i].read();//read ScreenChar {ascii_character: u8,color_code:ColorCode}
            assert_eq!(char::from(scr_char.ascii_character),c);
        }
    });

    serial_println!("[ok]");
}