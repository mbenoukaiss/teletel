# 📺 Teletel

It's 1997, and the Minitel is revolutionizing the way people connect. Since its launch in 1982, this pioneering device
has already become a global sensation, offering fast and compact ways for people to access everything: telephone
directories, video games, real-time communication with friends, family and even... well, let's just say it's versatile.
As teletel becomes a standard and asserts its domination over the internet, it is a crucial time for developers
to create innovative applications that will shape the future. Enter `teletel`, a Rust library that opens up new
possibilities for interacting with this device, enabling you to build powerful apps for this game-changing technology.

There's no Git, Rust, GitHub or Markdown yet, somehow, you're reading this. Magic? No, just the ✨Minitel✨.

## Getting started

You can either use a real Minitel device or spin up the built-in emulator (see below) to develop without hardware.
For a real Minitel, you will need a 5-pin DIN connector to USB cable. Connecting directly through UART to an
ESP32, Arduino or anything else is not yet supported.

First add the following to your `Cargo.toml` and change `minitel2` to `minitel1b` if you have a Minitel 1B:
```toml
[dependencies]
teletel = { version = "0.1.0", features = ["minitel2", "serial-terminal", "colors"] }
```

Once you plugged or the emulator started you can use the following code to send text to it:
```rust
#[macro_use]
extern crate teletel;

use std::error::Error;
use teletel::functions::{Clear, Foreground, Color, Repeat, SetCursor};
use teletel::terminal::{Optional, SerialTerminal, TcpTerminal, Tee};

fn main() -> Result<(), Box<dyn Error>> {
    let mut term = Tee::new(
        Optional::new(SerialTerminal::new("/dev/ttyUSB0", None)),
        Optional::new(TcpTerminal::emulator()),
    );

    send!(&mut term, [
        Clear,
        SetCursor(15, 11),
        "Hello ",
        Foreground(Color::Cyan, "World"),
        Foreground(Color::Red, Repeat('!', 3)),
    ])?;

    Ok(())
}
```

The `Tee` combinator lets you send to both a real Minitel and the emulator at
the same time. `Optional` makes either connection non-fatal if it's unavailable.

If running the code above on Linux gives you a permission error, add your user
to the `dialout` group and log out/in:
```bash
sudo adduser $USER dialout
```

## Emulator

The emulator is a standalone Bevy application that listens on TCP port 3615.
Any program using `TcpTerminal::emulator()` will connect to it automatically.

```bash
cargo run -p minitel-emulator
```

Once the emulator is running, run an example in another terminal:
```bash
cargo run -p example-hello
cargo run -p example-load-vdt
cargo run -p example-semigraphic
```

### Keyboard mapping

The emulator maps PC keys to Minitel function keys:

| PC key    | Minitel key     |
|-----------|-----------------|
| F1        | Envoi           |
| F2        | Retour          |
| F3        | Repetition      |
| F4        | Guide           |
| F5        | Annulation      |
| F6        | Sommaire        |
| F7        | Suite           |
| F8        | Connexion/Fin   |
| F9        | Correction      |
| Arrows    | Cursor movement |
| Backspace | Correction      |
| Enter     | Envoi           |
| Escape    | ESC             |

### Debug tools

Hold Ctrl and press a key to toggle debug overlays:

| Shortcut | Description                                                 |
|----------|-------------------------------------------------------------|
| Ctrl+G   | Grid overlay showing screen center and quarter divisions    |
| Ctrl+C   | Highlight the current protocol cursor position              |
| Ctrl+M   | Toggle mouse cell highlight (on by default)                 |
| Ctrl+B   | Cycle baud rate emulation: Unlimited, 300, 1200, 4800, 9600 |

The bottom-right corner shows current mouse position, cursor position, and baud
rate setting.

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
              Renames the <code>Color</code> enum variants from grayscale levels to color names (e.g. <code>Color::Cyan</code>
              instead of <code>Color::Gray80</code>). This is purely cosmetic and does not change the bytes sent to the terminal.
            </td>
        </tr>
        <tr>
            <td><code>serial-terminal</code></td>
            <td>disabled</td>
            <td>Enables communicating with the Minitel through a USB serial port.</td>
        </tr>
        <tr>
            <td><code>tcp-terminal</code></td>
            <td>disabled</td>
            <td>Enables communicating over TCP, used by the emulator.</td>
        </tr>
        <tr>
            <td><code>strict</code></td>
            <td>disabled</td>
            <td>
              When enabled, will make the parser return errors and stop consuming input when encountering
              an unknown or invalid sequence. If disabled a warning will be logged and parsing will continue.
            </td>
        </tr>
    </tbody>
</table>
