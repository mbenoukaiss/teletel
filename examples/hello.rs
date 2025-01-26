#[macro_use]
extern crate teletel;

use teletel::backend::{BaudRate, SerialBackend};
use teletel::{Beep, Blink, Clear, Color, Foreground, Move, Repeat};

fn main() {
    let mut port = SerialBackend::new("/dev/ttyUSB0", BaudRate::B9600);

    send!(&mut port, [
        Clear,
        Move(9, 11),
        Foreground(Color::Yellow, Repeat('H', 3)),
        Foreground(Color::Cyan, Repeat('E', 3)),
        Foreground(Color::Green, Repeat('L', 3)),
        Foreground(Color::Magenta, Repeat('L', 3)),
        Foreground(Color::Red, Repeat('O', 3)),
        " WORLD",
        Blink(Repeat('!', 2)),
        Beep,
    ]);
}
