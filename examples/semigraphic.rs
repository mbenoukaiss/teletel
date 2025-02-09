#[macro_use]
extern crate teletel;

use std::error::Error;
use std::thread::sleep;
use teletel::functions::{Big, Clear, Inverted, SetCursor, Repeat, SemiGraphic};
use teletel::protocol::ProtocolExtension;
use teletel::terminal::{SerialTerminal, WriteableTerminal};

/// Displays the Lumon droplet logo from
/// Severance on the minitel screen
fn main() -> Result<(), Box<dyn Error>> {
    let mut serial = SerialTerminal::new("/dev/ttyUSB0", None)?;

    draw_background(&mut serial)?;
    draw_droplet(&mut serial)?;

    send!(&mut serial, [
        SetCursor(15, 20),
        Big("Lumon"),
    ])?;

    Ok(())
}

fn draw_background(term: &mut dyn WriteableTerminal) -> Result<(), Box<dyn Error>> {
    send!(term, [
        Clear,

        SemiGraphic(list![
            SetCursor(10, 4),
            sg!(00/00/00),
            sg!(00/01/11),
            sg!(01/11/11),
            Repeat(sg!(11/11/11), 14),
            sg!(10/11/11),
            sg!(00/10/11),
            sg!(00/00/00),

            SetCursor(10, 5),
            SemiGraphic(list![
                sg!(01/01/11),
                Repeat(sg!(11/11/11), 18),
                sg!(10/10/11),
            ]),
        ]),
    ])?;

    for i in 1..12 {
        send!(term, [
            SetCursor(10, 5 + i),
            SemiGraphic(Repeat(sg!(11/11/11), 20)),
        ])?;
    }

    send!(term, [
        SemiGraphic(list![
            SetCursor(10, 16),
            sg!(11/01/01),
            Repeat(sg!(11/11/11), 18),
            sg!(11/10/10),

            SetCursor(10, 17),
            sg!(00/00/00),
            sg!(11/01/00),
            sg!(11/11/01),
            Repeat(sg!(11/11/11), 14),
            sg!(11/11/10),
            sg!(11/10/00),
            sg!(00/00/00),
        ]),
    ])?;

    Ok(())
}

fn draw_droplet(term: &mut dyn WriteableTerminal) -> Result<(), Box<dyn Error>> {
    send!(term, [
        Inverted(list![
            SemiGraphic(list![
                SetCursor(19, 6),
                sg!(10/10/00),
                sg!(01/01/00),

                SetCursor(18, 7),
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 2),
                sg!(11/01/01),

                SetCursor(17, 8),
                sg!(11/11/10),
                Repeat(sg!(00/00/00), 4),
                sg!(11/11/01),

                SetCursor(17, 9),
                sg!(10/00/00),
                Repeat(sg!(00/00/00), 4),
                sg!(01/00/00),

                SetCursor(16, 10),
                sg!(10/10/00),
                Repeat(sg!(00/00/00), 6),
                sg!(01/01/00),

                SetCursor(15, 11),
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 8),
                sg!(11/01/01),

                SetCursor(15, 12),
                Repeat(sg!(00/00/00), 10),

                SetCursor(15, 13),
                Repeat(sg!(00/00/00), 10),

                SetCursor(15, 14),
                sg!(10/10/11),
                Repeat(sg!(00/00/00), 8),
                sg!(01/01/11),

                SetCursor(16, 15),
                sg!(10/11/11),
                sg!(00/10/11),
                Repeat(sg!(00/00/11), 4),
                sg!(00/01/11),
                sg!(01/11/11),
            ]),
        ]),
    ])?;

    Ok(())
}