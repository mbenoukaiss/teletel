#[macro_use]
extern crate teletel;

use teletel::backend::{BaudRate, SerialBackend};
use teletel::protocol::codes::*;
use teletel::Blink;

fn main() {
    let mut port = SerialBackend::new("/dev/ttyUSB0", BaudRate::B9600);

    send!(port, [
        CLEAR,
        E_ACUTE,
        LOWER_OE,
        UPPER_OE,
        ESZETT,
        POUND,
        PARAGRAPH,
        BEEP,
        Blink("salut"),
        "mec",
    ]);
}
