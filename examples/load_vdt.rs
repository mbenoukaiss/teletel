#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::Videotex;
use teletel::terminal::{BaudRate, SerialTerminal};

fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = SerialTerminal::new("/dev/ttyUSB0", BaudRate::B9600)?;

    send!(&mut mt, [
        Videotex::from_path("examples/3615.vdt").unwrap(),
    ])?;

    Ok(())
}
