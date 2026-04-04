#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::Videotex;
use teletel::terminal::EmulatorTerminal;

fn main() -> Result<(), Box<dyn Error>> {
    let mut term = EmulatorTerminal::connect()?;

    send!(
        &mut term,
        [Videotex::from_path("examples/load-vdt/assets/3615.vdt").unwrap(),]
    )?;

    Ok(())
}
