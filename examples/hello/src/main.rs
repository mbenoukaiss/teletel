#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::drawing::RectangleOutline;
use teletel::functions::{Beep, Blink, Clear, Color, Foreground, Repeat, SetCursor};
use teletel::terminal::{Optional, ReadableTerminal, SerialTerminal, TcpTerminal, Tee};

fn main() -> Result<(), Box<dyn Error>> {
    let mut term = Tee::new(
        Optional::new(SerialTerminal::new("/dev/ttyUSB0", None)),
        Optional::new(TcpTerminal::emulator()),
    );

    send!(
        &mut term,
        [
            Clear,
            SetCursor(9, 11),
            Foreground(
                Color::Cyan,
                list![
                    Repeat('H', 3),
                    Repeat('E', 3),
                    Repeat('L', 6),
                    Repeat('O', 3),
                    " WORLD",
                    Repeat('!', 2),
                ]
            ),
            SetCursor(8, 10),
            Blink(RectangleOutline(25, 3, RectangleOutline::OUT)),
            Beep,
        ]
    )?;

    println!(
        "read from keyboard: {}",
        String::from_utf8(term.read_until_enter()?)?
    );

    Ok(())
}
