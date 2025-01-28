#[macro_use]
extern crate teletel;

use teletel::receiver::{BaudRate, SerialReceiver};
use teletel::Videotex;

fn main() {
    let mut port = SerialReceiver::new("/dev/ttyUSB0", BaudRate::B9600);

    send!(&mut port, [
        Videotex::from_path("examples/3615.vdt").unwrap(),
    ]);
}
