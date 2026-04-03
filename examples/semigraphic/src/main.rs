#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::{Big, Clear, Inverted, Repeat, SemiGraphic, SetCursor};
use teletel::terminal::{SerialTerminal, WriteableTerminal};

/// Displays the Lumon droplet logo from
/// Severance on the minitel screen
fn main() -> Result<(), Box<dyn Error>> {
    let mut serial = SerialTerminal::new("/dev/ttyUSB0", None)?;

    draw_background(&mut serial)?;
    draw_droplet(&mut serial)?;

    send!(&mut serial, [SetCursor(15, 20), Big("Lumon"),])?;

    Ok(())
}

fn draw_background(term: &mut dyn WriteableTerminal) -> Result<(), Box<dyn Error>> {
    send!(
        term,
        [
            Clear,
            SemiGraphic(list![
                SetCursor(10, 4),
                sg!(000000),
                sg!(000111),
                sg!(011111),
                Repeat(sg!(111111), 14),
                sg!(101111),
                sg!(001011),
                sg!(000000),
                SetCursor(10, 5),
                SemiGraphic(list![sg!(010111), Repeat(sg!(111111), 18), sg!(101011),]),
            ]),
        ]
    )?;

    for i in 1..12 {
        send!(
            term,
            [SetCursor(10, 5 + i), SemiGraphic(Repeat(sg!(111111), 20)),]
        )?;
    }

    send!(
        term,
        [SemiGraphic(list![
            SetCursor(10, 16),
            sg!(110101),
            Repeat(sg!(111111), 18),
            sg!(111010),
            SetCursor(10, 17),
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
            SetCursor(19, 6),
            sg!(101000),
            sg!(010100),
            SetCursor(18, 7),
            sg!(111010),
            Repeat(sg!(000000), 2),
            sg!(110101),
            SetCursor(17, 8),
            sg!(111110),
            Repeat(sg!(000000), 4),
            sg!(111101),
            SetCursor(17, 9),
            sg!(100000),
            Repeat(sg!(000000), 4),
            sg!(010000),
            SetCursor(16, 10),
            sg!(101000),
            Repeat(sg!(000000), 6),
            sg!(010100),
            SetCursor(15, 11),
            sg!(111010),
            Repeat(sg!(000000), 8),
            sg!(110101),
            SetCursor(15, 12),
            Repeat(sg!(000000), 10),
            SetCursor(15, 13),
            Repeat(sg!(000000), 10),
            SetCursor(15, 14),
            sg!(101011),
            Repeat(sg!(000000), 8),
            sg!(010111),
            SetCursor(16, 15),
            sg!(101111),
            sg!(001011),
            Repeat(sg!(000011), 4),
            sg!(000111),
            sg!(011111),
        ]),]),]
    )?;

    Ok(())
}
