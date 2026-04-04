#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::{Big, Clear, Inverted, Repeat, SemiGraphic, SetCursor};
use teletel::terminal::{Optional, SerialTerminal, TcpTerminal, Dual, WriteableTerminal};

/// Displays the Lumon droplet logo from
/// Severance on the minitel screen
fn main() -> Result<(), Box<dyn Error>> {
    let mut term = Dual::new(
        Optional::new(SerialTerminal::new("/dev/ttyUSB0", None)),
        Optional::new(TcpTerminal::emulator()),
    );

    draw_background(&mut term)?;
    draw_droplet(&mut term)?;

    send!(&mut term, [SetCursor(16, 20), Big("Lumon"),])?;

    Ok(())
}

fn draw_background(term: &mut dyn WriteableTerminal) -> Result<(), Box<dyn Error>> {
    send!(
        term,
        [
            Clear,
            SemiGraphic(list![
                SetCursor(11, 4),
                sg!(000000),
                sg!(000111),
                sg!(011111),
                Repeat(sg!(111111), 14),
                sg!(101111),
                sg!(001011),
                sg!(000000),
                SetCursor(11, 5),
                SemiGraphic(list![sg!(010111), Repeat(sg!(111111), 18), sg!(101011),]),
            ]),
        ]
    )?;

    for i in 1..12 {
        send!(
            term,
            [SetCursor(11, 5 + i), SemiGraphic(Repeat(sg!(111111), 20)),]
        )?;
    }

    send!(
        term,
        [SemiGraphic(list![
            SetCursor(11, 16),
            sg!(110101),
            Repeat(sg!(111111), 18),
            sg!(111010),
            SetCursor(11, 17),
            sg!(000000),
            sg!(110100),
            sg!(111101),
            Repeat(sg!(111111), 14),
            sg!(111110),
            sg!(111000),
            sg!(000000),
        ]),]
    )?;

    Ok(())
}

fn draw_droplet(term: &mut dyn WriteableTerminal) -> Result<(), Box<dyn Error>> {
    send!(
        term,
        [Inverted(list![SemiGraphic(list![
            SetCursor(20, 6),
            sg!(101000),
            sg!(010100),
            SetCursor(19, 7),
            sg!(111010),
            Repeat(sg!(000000), 2),
            sg!(110101),
            SetCursor(18, 8),
            sg!(111110),
            Repeat(sg!(000000), 4),
            sg!(111101),
            SetCursor(18, 9),
            sg!(100000),
            Repeat(sg!(000000), 4),
            sg!(010000),
            SetCursor(17, 10),
            sg!(101000),
            Repeat(sg!(000000), 6),
            sg!(010100),
            SetCursor(16, 11),
            sg!(111010),
            Repeat(sg!(000000), 8),
            sg!(110101),
            SetCursor(16, 12),
            Repeat(sg!(000000), 10),
            SetCursor(16, 13),
            Repeat(sg!(000000), 10),
            SetCursor(16, 14),
            sg!(101011),
            Repeat(sg!(000000), 8),
            sg!(010111),
            SetCursor(17, 15),
            sg!(101111),
            sg!(001011),
            Repeat(sg!(000011), 4),
            sg!(000111),
            sg!(011111),
        ]),]),]
    )?;

    Ok(())
}
