#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::{BaudRate, Minitel};
use teletel::functions::{Beep, Blink, Clear, Color, Foreground, Repeat, SetCursor};
use teletel::drawing::RectangleOutline;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = Minitel::serial("/dev/ttyUSB0", BaudRate::B9600)?;

    println!("{:#04X}", sg!(10/10/10));

    send!(&mut mt, [
        Clear,
        SetCursor(9, 11),
        Foreground(Color::Gray80, list![
            Repeat('H', 3),
            Repeat('E', 3),
            Repeat('L', 6),
            Repeat('O', 3),
            " WORLD",
            Repeat('!', 2),
        ]),
        SetCursor(8, 10),
        Blink(RectangleOutline(25, 3, RectangleOutline::OUT)),
        Beep,
    ])?;

    println!("read from keyboard: {}", String::from_utf8(mt.read_until_enter()?)?);

    Ok(())
}
