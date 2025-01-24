use crate::backend::{BaudRate, SerialBackend};
use crate::codes::*;

#[macro_use]
mod utils;

mod backend;
mod codes;

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
        ESC,
         BLINK,
        "salut",
        ESC,
        STILL,
        "mec",
    ]);
}
