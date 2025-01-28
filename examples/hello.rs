#[macro_use]
extern crate teletel;

use teletel::receiver::{BaudRate, SerialReceiver};
use teletel::{Beep, Blink, Clear, Color, Foreground, SetCursor, Repeat, Error};

fn main() -> Result<(), Error> {
    let mut port = SerialReceiver::new("/dev/ttyUSB0", BaudRate::B9600)?;

    send!(&mut port, [
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

    Ok(())
}
