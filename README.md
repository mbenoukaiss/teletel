# 📺 Teletel

It’s 1997, and the Minitel is revolutionizing the way people connect. Since its launch in 1982, this pioneering device
has already become a global sensation, offering fast and compact ways for people to access everything: telephone 
directories, video games, real-time communication with friends, family and even... well, let’s just say it’s versatile.
As teletel becomes a standard and asserts its domination over the internet, it is a crucial time for developers   
to create innovative applications that will shape the future. Enter `teletel`, a Rust library that opens up new 
possibilities for interacting with this device, enabling you to build powerful apps for this game-changing technology.

There’s no Git, Rust, GitHub or Markdown yet, somehow, you’re reading this. Magic? No, just the ✨Minitel✨

## Getting started
You will need a Minitel device and some way to communicate with it. You can either make or buy a 5-pin DIN connector 
to USB cable specifically for the minitel. Connecting it directly through UART to an ESP32, Arduino or anything else
is not yet supported.

First add the following to your `Cargo.toml` and change `minitel2` to `minitel1b` if you have a Minitel 1B:
```toml
[dependencies]
teletel = { version = "???", features = ["minitel2", "serial"] }
```

Once you plugged the Minitel you can use the following code to send text to it:
```rust
#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::{Clear, Foreground, Color, Repeat, SetCursor};
use teletel::terminal::SerialTerminal;

fn main() -> Result<(), Box<dyn Error>> {
    //change path to match your setup, you can leave None for baudrate to scan for correct rate
    let mut port = SerialTerminal::new("/dev/ttyUSB0", None)?;

    send!(&mut port, [
        Clear,
        SetCursor(15, 11),
        "Hello",
        Foreground(Color::Gray90, "World"),
        Foreground(Color::Gray50, Repeat('!', 3)),
    ])?;

    Ok(())
}
```

If running the code above gives you a permission error, you can add your user to the `dialout` group with the 
following command, don't forget to log out and log back in after running it:
```bash
sudo adduser $USER dialout
```

## Features

<table>
    <thead>
        <tr>
            <th>Feature</th>
            <th>Default</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>minitel2</code></td>
            <td>disabled</td>
            <td>Compatibility mode for the Minitel 2.</td>
        </tr>
        <tr>
            <td><code>minitel1b</code></td>
            <td>disabled</td>
            <td>
              Compatibility mode for the Minitel 1B, it is not strictly necessary to enable this feature when 
              dealing with a Minitel 1B, but it disables some features that are not available on the Minitel 
              1B and avoids trying to use them without knowing.
            </td>
        </tr>
        <tr>
            <td><code>colors</code></td>
            <td>disabled</td>
            <td>
              Changes the `Color` enum variants to be the 8 colors available on the versions of Minitel that 
              support colors instead of grayscale.</td>
        </tr>
        <tr>
            <td><code>serial</code></td>
            <td>disabled</td>
            <td>Enables communicating with the Minitel through a USB port.</td>
        </tr>
        <tr>
            <td><code>strict</code></td>
            <td>disabled</td>
            <td>
              When enabled, will make the parser return errors and stop consuming input and sending them to 
              the terminal when encountering an unknown or invalid sequence. If disabled a warning will be
              logged and the input will be sent to the terminal.
            </td>
        </tr>
    </tbody>
</table>