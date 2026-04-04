use crate::config::{EmulatorConfig, SCREEN_COLUMNS, SCREEN_ROWS};
use crate::glyphs::GlyphCache;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use teletel_protocol::parser::Context;

use super::beep::BeepSound;
use super::debug::DebugState;
use super::palette::{cell_index, cell_position, palette_for};
use super::{
    TerminalEntities, TerminalState, TransportResource,
    BACKGROUND_Z, CURSOR_Z, FOREGROUND_Z,
};

pub(super) fn pump_transport_input(
    mut commands: Commands,
    time: Res<Time>,
    mut transport: ResMut<TransportResource>,
    mut terminal: ResMut<TerminalState>,
    debug_state: Res<DebugState>,
    beep_sound: Res<BeepSound>,
) {
    // read all available bytes into the pending buffer
    loop {
        match transport.read_available() {
            Ok(Some(bytes)) => terminal.pending_bytes.extend(bytes),
            Ok(None) => break,
            Err(err) => {
                error!("failed to read emulator PTY: {err}");
                break;
            }
        }
    }

    // feed bytes to parser, respecting baud rate throttling
    if let Some(bps) = debug_state.baud_rate.bytes_per_second() {
        terminal.byte_budget += time.delta_secs() * bps;
        let to_consume = (terminal.byte_budget as usize).min(terminal.pending_bytes.len());
        for _ in 0..to_consume {
            if let Some(byte) = terminal.pending_bytes.pop_front() {
                if let Err(err) = terminal.parser.consume(byte) {
                    error!("failed to consume PTY byte {byte:#04X}: {err}");
                }
            }
        }
        terminal.byte_budget -= to_consume as f32;
    } else {
        let bytes: Vec<u8> = terminal.pending_bytes.drain(..).collect();
        for byte in bytes {
            if let Err(err) = terminal.parser.consume(byte) {
                error!("failed to consume PTY byte {byte:#04X}: {err}");
            }
        }
        terminal.byte_budget = 0.0;
    }

    // send protocol response bytes back to the remote host
    let response = terminal.parser.take_response();
    if !response.is_empty() {
        if let Err(err) = transport.write_all(&response) {
            error!("failed to write protocol response: {err}");
        }
    }

    if terminal.parser.take_beep() {
        commands.spawn(AudioPlayer(beep_sound.0.clone()));
    }
}

pub(super) fn capture_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut transport: ResMut<TransportResource>,
) {
    // when ctrl is held, debug shortcuts are active, don't send keys to terminal
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl_held {
        return;
    }

    let mut bytes = Vec::new();

    for event in keyboard_inputs.read() {
        if !event.state.is_pressed() {
            continue;
        }

        if let Some(mapped) = map_named_key(event.key_code) {
            bytes.extend_from_slice(mapped);
            continue;
        }

        if let Some(text) = &event.text {
            bytes.extend(text.chars().filter_map(map_character));
        }
    }

    if !bytes.is_empty() {
        if let Err(err) = transport.write_all(&bytes) {
            error!("failed to write keyboard bytes to PTY: {err}");
        }
    }
}

pub(super) fn advance_blink_phase(time: Res<Time>, mut terminal: ResMut<TerminalState>) {
    if terminal.blink_timer.tick(time.delta()).just_finished() {
        terminal.blink_visible = !terminal.blink_visible;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_terminal(
    terminal: Res<TerminalState>,
    config: Res<EmulatorConfig>,
    entities: Res<TerminalEntities>,
    mut glyph_cache: ResMut<GlyphCache>,
    mut images: ResMut<Assets<Image>>,
    mut sprites: Query<&mut Sprite>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
) {
    let ctx = terminal.parser.ctx();
    let coverage = coverage_map(ctx);

    for y in 1..=SCREEN_ROWS {
        for x in 1..=SCREEN_COLUMNS {
            let index = cell_index(x, y);
            let cell = ctx.grid.cell(x, y);
            let owner = coverage[index];
            let background_entity = entities.cells[index].background;
            let foreground_entity = entities.cells[index].foreground;

            let Some(owner_index) = owner else {
                if let Ok(mut visible) = visibility.get_mut(background_entity) {
                    *visible = Visibility::Hidden;
                }
                if let Ok(mut visible) = visibility.get_mut(foreground_entity) {
                    *visible = Visibility::Hidden;
                }
                continue;
            };

            if owner_index != index {
                if let Ok(mut visible) = visibility.get_mut(background_entity) {
                    *visible = Visibility::Hidden;
                }
                if let Ok(mut visible) = visibility.get_mut(foreground_entity) {
                    *visible = Visibility::Hidden;
                }
                continue;
            }

            let owner_cell = cell;
            let (fg, bg) = palette_for(owner_cell, config.colors);
            let width = if owner_cell.attributes.double_width { 2.0 } else { 1.0 };
            let height = if owner_cell.attributes.double_height { 2.0 } else { 1.0 };
            let position = cell_position(&config, x, y);

            if let Ok(mut sprite) = sprites.get_mut(background_entity) {
                sprite.color = bg;
                sprite.custom_size = Some(Vec2::new(
                    config.cell_size.x * width,
                    config.cell_size.y * height,
                ));
            }
            if let Ok(mut transform) = transforms.get_mut(background_entity) {
                transform.translation = position.extend(BACKGROUND_Z);
            }
            if let Ok(mut visible) = visibility.get_mut(background_entity) {
                *visible = Visibility::Inherited;
            }

            let show_foreground = owner_cell.content != '\0'
                && owner_cell.content != ' '
                && (!owner_cell.attributes.blinking || terminal.blink_visible)
                && (!owner_cell.attributes.mask || !ctx.screen_mask);

            if let Ok(mut visible) = visibility.get_mut(foreground_entity) {
                *visible = if show_foreground {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if !show_foreground {
                continue;
            }

            if let Some(handle) = glyph_cache.glyph_for(&mut images, owner_cell) {
                if let Ok(mut sprite) = sprites.get_mut(foreground_entity) {
                    sprite.image = handle;
                    sprite.color = fg;
                    sprite.custom_size = Some(Vec2::new(
                        config.cell_size.x * width,
                        config.cell_size.y * height,
                    ));
                }
                if let Ok(mut transform) = transforms.get_mut(foreground_entity) {
                    transform.translation = position.extend(FOREGROUND_Z);
                }
            }
        }
    }

    let cursor_entity = entities.cursor;
    let cursor_visible = ctx.visible_cursor && terminal.blink_visible;
    if let Ok(mut visible) = visibility.get_mut(cursor_entity) {
        *visible = if cursor_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if cursor_visible {
        if let Ok(mut sprite) = sprites.get_mut(cursor_entity) {
            sprite.custom_size = Some(config.cell_size);
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.35);
        }
        if let Ok(mut transform) = transforms.get_mut(cursor_entity) {
            transform.translation =
                cell_position(&config, ctx.cursor_x, ctx.cursor_y).extend(CURSOR_Z);
        }
    }
}

fn coverage_map(ctx: &Context) -> Vec<Option<usize>> {
    let mut coverage = vec![None; (SCREEN_COLUMNS as usize) * (SCREEN_ROWS as usize)];

    for y in 1..=SCREEN_ROWS {
        for x in 1..=SCREEN_COLUMNS {
            let index = cell_index(x, y);
            let cell = ctx.grid.cell(x, y);

            let width = if cell.attributes.double_width { 2 } else { 1 };
            let height = if cell.attributes.double_height { 2 } else { 1 };

            for dx in 0..width {
                for dy in 0..height {
                    let target_x = x + dx;
                    let target_y = y.saturating_sub(dy);

                    if target_x == 0
                        || target_x > SCREEN_COLUMNS
                        || target_y == 0
                        || target_y > SCREEN_ROWS
                    {
                        continue;
                    }

                    coverage[cell_index(target_x, target_y)] = Some(index);
                }
            }
        }
    }

    coverage
}

fn map_character(character: char) -> Option<u8> {
    if character.is_control() {
        return None;
    }

    if character.is_ascii() {
        Some(character as u8)
    } else {
        None
    }
}

fn map_named_key(code: KeyCode) -> Option<&'static [u8]> {
    use teletel_protocol::codes::keyboard;

    if code == KeyCode::Enter || code == KeyCode::NumpadEnter {
        Some(&[0x0D])
    } else if code == KeyCode::Space {
        Some(&[0x20])
    } else if code == KeyCode::Tab {
        Some(&[0x09])
    } else if code == KeyCode::Escape {
        Some(&[0x1B])
    } else if code == KeyCode::Backspace || code == KeyCode::Delete {
        Some(&keyboard::CORRECTION)
    } else if code == KeyCode::ArrowUp {
        Some(&[0x0B])
    } else if code == KeyCode::ArrowDown {
        Some(&[0x0A])
    } else if code == KeyCode::ArrowLeft {
        Some(&[0x08])
    } else if code == KeyCode::ArrowRight {
        Some(&[0x09])
    } else if code == KeyCode::F1 {
        Some(&keyboard::ENVOI)
    } else if code == KeyCode::F2 {
        Some(&keyboard::RETOUR)
    } else if code == KeyCode::F3 {
        Some(&keyboard::REPETITION)
    } else if code == KeyCode::F4 {
        Some(&keyboard::GUIDE)
    } else if code == KeyCode::F5 {
        Some(&keyboard::ANNULATION)
    } else if code == KeyCode::F6 {
        Some(&keyboard::SOMMAIRE)
    } else if code == KeyCode::F7 {
        Some(&keyboard::SUITE)
    } else if code == KeyCode::F8 {
        Some(&keyboard::CONNEXION_FIN)
    } else if code == KeyCode::F9 {
        Some(&keyboard::CORRECTION)
    } else {
        None
    }
}
