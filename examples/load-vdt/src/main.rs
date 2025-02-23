#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::Videotex;
use teletel::terminal::SerialTerminal;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = SerialTerminal::new("/dev/ttyUSB0", None)?;

    send!(&mut mt, [
        Videotex::from_path("examples/load-vdt/assets/3615.vdt").unwrap(),
    ])?;

    Ok(())
}
