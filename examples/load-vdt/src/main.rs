#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::Videotex;
use teletel::terminal::{Optional, SerialTerminal, TcpTerminal, Tee};

fn main() -> Result<(), Box<dyn Error>> {
    let mut term = Tee::new(
        Optional::new(SerialTerminal::new("/dev/ttyUSB0", None)),
        Optional::new(TcpTerminal::emulator()),
    );

    send!(
        &mut term,
        [Videotex::from_path("examples/load-vdt/assets/3615.vdt").unwrap(),]
    )?;

    Ok(())
}
