#[macro_use]
extern crate teletel;

use teletel::receiver::{TeletelReceiver, BaudRate, SerialReceiver};
use teletel::{Big, Clear, Inverted, Move, Repeat, SemiGraphic};

/// Displays the Lumon droplet logo from
/// Severance on the minitel screen
fn main() {
    let mut port = SerialReceiver::new("/dev/ttyUSB0", BaudRate::B9600);

    draw_background(&mut port);
    draw_droplet(&mut port);

    send!(&mut port, [
        Move(15, 18),
        Big("Lumon"),
    ]);
}

fn draw_background(port: &mut dyn TeletelReceiver) {
    send!(port, [
        Clear,

        Move(10, 2),
        SemiGraphic(from![
            sg!(00/00/00),
            sg!(00/01/11),
            sg!(01/11/11),
            Repeat(sg!(11/11/11), 14),
            sg!(10/11/11),
            sg!(00/10/11),
            sg!(00/00/00),
        ]),

        Move(10, 3),
        SemiGraphic(from![
            sg!(01/01/11),
            Repeat(sg!(11/11/11), 18),
            sg!(10/10/11),
        ]),
    ]);

    for i in 1..12 {
        send!(port, [
            Move(10, 3 + i),
            SemiGraphic(Repeat(sg!(11/11/11), 20)),
        ]);
    }

    send!(port, [
        Move(10, 14),
        SemiGraphic(from![
            sg!(11/01/01),
            Repeat(sg!(11/11/11), 18),
            sg!(11/10/10),
        ]),

        Move(10, 15),
        SemiGraphic(from![
            sg!(00/00/00),
            sg!(11/01/00),
            sg!(11/11/01),
            Repeat(sg!(11/11/11), 14),
            sg!(11/11/10),
            sg!(11/10/00),
            sg!(00/00/00),
        ]),
    ]);
}

fn draw_droplet(port: &mut dyn TeletelReceiver) {
    send!(port, [
        Inverted(from![
            Move(19, 4),
            SemiGraphic([
                sg!(10/10/00),
                sg!(01/01/00),
            ]),

            Move(18, 5),
            SemiGraphic(from![
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 2),
                sg!(11/01/01),
            ]),

            Move(17, 6),
            SemiGraphic(from![
                sg!(11/11/10),
                Repeat(sg!(00/00/00), 4),
                sg!(11/11/01),
            ]),


            Move(17, 7),
            SemiGraphic(from![
                sg!(10/00/00),
                Repeat(sg!(00/00/00), 4),
                sg!(01/00/00),
            ]),

            Move(16, 8),
            SemiGraphic(from![
                sg!(10/10/00),
                Repeat(sg!(00/00/00), 6),
                sg!(01/01/00),
            ]),

            Move(15, 9),
            SemiGraphic(from![
                sg!(11/10/10),
                Repeat(sg!(00/00/00), 8),
                sg!(11/01/01),
            ]),

            Move(15, 10),
            SemiGraphic(from![
                Repeat(sg!(00/00/00), 10),
            ]),

            Move(15, 11),
            SemiGraphic(from![
                Repeat(sg!(00/00/00), 10),
            ]),

            Move(15, 12),
            SemiGraphic(from![
                sg!(10/10/11),
                Repeat(sg!(00/00/00), 8),
                sg!(01/01/11),
            ]),

            Move(16, 13),
            SemiGraphic(from![
                sg!(10/11/11),
                sg!(00/10/11),
                Repeat(sg!(00/00/11), 4),
                sg!(00/01/11),
                sg!(01/11/11),
            ]),
        ]),
    ]);
}