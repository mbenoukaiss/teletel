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
        Foreground(Color::Yellow, Repeat('H', 3)),
        Foreground(Color::Cyan, Repeat('E', 3)),
        Foreground(Color::Green, Repeat('L', 3)),
        Foreground(Color::Magenta, Repeat('L', 3)),
        Foreground(Color::Red, Repeat('O', 3)),
        " WORLD",
        Blink(Repeat('!', 2)),
        Beep,
    ])?;

    println!("read from keyboard : {}", String::from_utf8(mt.read_until_enter()?)?);

    Ok(())
}
