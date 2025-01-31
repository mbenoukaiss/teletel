#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::{BaudRate, Minitel};
use teletel::functions::Videotex;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = Minitel::serial("/dev/ttyUSB0", BaudRate::B9600)?;

    send!(&mut mt, [
        Videotex::from_path("examples/3615.vdt").unwrap(),
    ])?;

    Ok(())
}
