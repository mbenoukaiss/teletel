mod config;
mod glyphs;
mod render;
mod transport;

use crate::config::EmulatorConfig;
use crate::render::{EmulatorPlugin, TransportResource};
use crate::transport::{TcpTransport, EMULATOR_PORT};
use bevy::prelude::*;
use bevy::window::WindowPlugin;

fn main() {
    let config = EmulatorConfig::default();
    let transport = TcpTransport::bind().expect("failed to bind TCP transport");
    println!("Minitel emulator listening on 127.0.0.1:{EMULATOR_PORT}");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(config.clone())
        .insert_resource(TransportResource(transport))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: config.title.clone(),
                        resolution: config.window_resolution.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(EmulatorPlugin)
        .run();
}
