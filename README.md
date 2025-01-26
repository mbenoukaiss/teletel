# Teletel

It’s 1997, and the Minitel is revolutionizing the way people connect. Since its launch in 1982, this pioneering device
has already become a global sensation, offering fast and compact ways for people to access everything : telephone 
directories, video games, real-time communication with friends, family and even... well, let’s just say it’s versatile.
As the Minitel continues to gain market shares against the internet, it is a crucial time for developers to create 
innovative applications that will shape the future. Enter `teletel`, a Rust library that opens up new possibilities
for interacting with this device, enabling you to build powerful apps for this game-changing technology.

There’s no Git, Rust, GitHub or Markdown yet, somehow, you’re reading this. Magic? No, just the ✨Minitel✨

## Getting started
You will need a Minitel device and some way to communicate with it. You can either make or buy a 5-pin DIN connector 
to USB cable specifically for the minitel. Connecting it directly through UART to an ESP32, Arduino or anything else
is not yet supported.

Once you plug the Minitel you can use the following code to send text to it:
```rust
#[macro_use]
extern crate teletel;

use teletel::receiver::{BaudRate, SerialReceiver};
use teletel::Clear;

fn main() {
    //change path and baudrate to match your setup
    let mut port = SerialReceiver::new("/dev/ttyUSB0", BaudRate::B9600);

    send!(&mut port, [
        Clear,
        "Hello World !",
    ]);
}
```

If running the code above gives you a permission error, you can add your user to the `dialout` group with the 
following command, don't forget to log out and log back in after running it:
```bash
sudo adduser $USER dialout
```
