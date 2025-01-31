#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::{BaudRate, Minitel};
use teletel::functions::{Big, Clear, Inverted, SetCursor, Repeat, SemiGraphic};

/// Displays the Lumon droplet logo from
/// Severance on the minitel screen
fn main() -> Result<(), Box<dyn Error>> {
    let mut mt = Minitel::serial("/dev/ttyUSB0", BaudRate::B9600)?;

    draw_background(&mut mt)?;
    draw_droplet(&mut mt)?;

    send!(&mut mt, [
        SetCursor(15, 18),
        Big("Lumon"),
    ])?;

    Ok(())
}

fn draw_background(mt: &mut Minitel) -> Result<(), Box<dyn Error>> {
    send!(mt, [
        Clear,

        SemiGraphic(list![
            SetCursor(10, 2),
            sg!(00/00/00),
            sg!(00/01/11),
            sg!(01/11/11),
            Repeat(sg!(11/11/11), 14),
            sg!(10/11/11),
            sg!(00/10/11),
            sg!(00/00/00),

            SetCursor(10, 3),
            SemiGraphic(list![
                sg!(01/01/11),
                Repeat(sg!(11/11/11), 18),
                sg!(10/10/11),
            ]),
        ]),
    ])?;

    for i in 1..12 {
        send!(mt, [
            SetCursor(10, 3 + i),
            SemiGraphic(Repeat(sg!(11/11/11), 20)),
        ])?;
    }

    send!(mt, [
        SemiGraphic(list![
            SetCursor(10, 14),
            sg!(11/01/01),
            Repeat(sg!(11/11/11), 18),
            sg!(11/10/10),

            SetCursor(10, 15),
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

fn draw_droplet(mt: &mut Minitel) -> Result<(), Box<dyn Error>> {
    send!(mt, [
        Inverted(list![
            SemiGraphic(list![
                SetCursor(19, 4),
                sg!(10/10/00),
                sg!(01/01/00),

                SetCursor(18, 5),
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 2),
                sg!(11/01/01),

                SetCursor(17, 6),
                sg!(11/11/10),
                Repeat(sg!(00/00/00), 4),
                sg!(11/11/01),

                SetCursor(17, 7),
                sg!(10/00/00),
                Repeat(sg!(00/00/00), 4),
                sg!(01/00/00),

                SetCursor(16, 8),
                sg!(10/10/00),
                Repeat(sg!(00/00/00), 6),
                sg!(01/01/00),

                SetCursor(15, 9),
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 8),
                sg!(11/01/01),

                SetCursor(15, 10),
                Repeat(sg!(00/00/00), 10),

                SetCursor(15, 11),
                Repeat(sg!(00/00/00), 10),

                SetCursor(15, 12),
                sg!(10/10/11),
                Repeat(sg!(00/00/00), 8),
                sg!(01/01/11),

                SetCursor(16, 13),
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