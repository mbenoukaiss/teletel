#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::{Beep, Blink, Clear, Color, Foreground, Repeat, SetCursor};
use teletel::{BaudRate, Minitel};

fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = Minitel::serial("/dev/ttyUSB0", BaudRate::B9600)?;

    send!(&mut mt, [
        Clear,
        SetCursor(9, 11),
        Foreground(Color::Gray90, Repeat('H', 3)),
        Foreground(Color::Gray80, Repeat('E', 3)),
        Foreground(Color::Gray70, Repeat('L', 3)),
        Foreground(Color::Gray60, Repeat('L', 3)),
        Foreground(Color::Gray50, Repeat('O', 3)),
        " WORLD",
        Blink(Repeat('!', 2)),
        Beep,
    ])?;

    println!("read from keyboard: {}", String::from_utf8(mt.read_until_enter()?)?);

    Ok(())
}
