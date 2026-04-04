mod beep;
mod debug;
mod palette;
mod setup;
mod terminal;

use crate::transport::TcpTransport;
use bevy::prelude::*;
use std::collections::VecDeque;
use teletel_protocol::parser::{DisplayComponent, Parser};

const BLINK_INTERVAL_SECS: f32 = 0.5;
const CURSOR_Z: f32 = 3.0;
const DEBUG_HIGHLIGHT_Z: f32 = 2.0;
const BACKGROUND_Z: f32 = 0.0;
const FOREGROUND_Z: f32 = 1.0;

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::glyphs::GlyphCache>()
            .init_resource::<TerminalState>()
            .init_resource::<debug::DebugState>()
            .add_systems(Startup, (setup::setup_terminal, beep::setup_beep_sound))
            .add_systems(
                Update,
                (
                    terminal::pump_transport_input,
                    debug::handle_debug_shortcuts,
                    terminal::capture_keyboard_input,
                    terminal::advance_blink_phase,
                    terminal::render_terminal,
                    debug::update_debug_overlay,
                )
                    .chain(),
            );
    }
}

// --- Shared resources and components ---

#[derive(Resource, Deref, DerefMut)]
pub struct TransportResource(pub TcpTransport);

#[derive(Resource)]
pub struct TerminalState {
    pub(crate) parser: Parser,
    pub(crate) blink_visible: bool,
    blink_timer: Timer,
    pub(crate) pending_bytes: VecDeque<u8>,
    pub(crate) byte_budget: f32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            parser: Parser::new(DisplayComponent::VGP5),
            blink_visible: true,
            blink_timer: Timer::from_seconds(BLINK_INTERVAL_SECS, TimerMode::Repeating),
            pending_bytes: VecDeque::new(),
            byte_budget: 0.0,
        }
    }
}

#[derive(Resource)]
struct TerminalEntities {
    cells: Vec<CellEntities>,
    cursor: Entity,
}

#[derive(Clone, Copy)]
struct CellEntities {
    background: Entity,
    foreground: Entity,
}

#[derive(Component)]
struct CursorOverlay;
