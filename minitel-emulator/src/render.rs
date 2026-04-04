use crate::config::{EmulatorConfig, SCREEN_COLUMNS, SCREEN_ROWS};
use crate::glyphs::GlyphCache;
use crate::transport::TcpTransport;

// Bevy Resource wrapper for TcpTransport
#[derive(Resource, Deref, DerefMut)]
pub struct TransportResource(pub TcpTransport);
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use teletel_protocol::parser::{Cell, Context, DisplayComponent, Parser};

const BLINK_INTERVAL_SECS: f32 = 0.5;
const CURSOR_Z: f32 = 3.0;
const BACKGROUND_Z: f32 = 0.0;
const FOREGROUND_Z: f32 = 1.0;

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphCache>()
            .init_resource::<TerminalState>()
            .add_systems(Startup, setup_terminal)
            .add_systems(
                Update,
                (
                    pump_transport_input,
                    capture_keyboard_input,
                    advance_blink_phase,
                    render_terminal,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
pub struct TerminalState {
    parser: Parser,
    blink_visible: bool,
    blink_timer: Timer,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            parser: Parser::new(DisplayComponent::VGP5),
            blink_visible: true,
            blink_timer: Timer::from_seconds(BLINK_INTERVAL_SECS, TimerMode::Repeating),
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

fn setup_terminal(mut commands: Commands, config: Res<EmulatorConfig>) {
    let screen_size = config.screen_size();

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: screen_size.y + config.cell_size.y * 2.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let mut cells = Vec::with_capacity((SCREEN_COLUMNS as usize) * (SCREEN_ROWS as usize));
    for y in 1..=SCREEN_ROWS {
        for x in 1..=SCREEN_COLUMNS {
            let position = cell_position(&config, x, y);
            let background = commands
                .spawn((
                    Sprite::from_color(Color::BLACK, config.cell_size),
                    Transform::from_translation(position.extend(BACKGROUND_Z)),
                    Anchor::BOTTOM_LEFT,
                ))
                .id();
            let foreground = commands
                .spawn((
                    Sprite::from_color(Color::NONE, config.cell_size),
                    Transform::from_translation(position.extend(FOREGROUND_Z)),
                    Anchor::BOTTOM_LEFT,
                    Visibility::Hidden,
                ))
                .id();

            cells.push(CellEntities {
                background,
                foreground,
            });
        }
    }

    let cursor = commands
        .spawn((
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.35), config.cell_size),
            Transform::from_translation(Vec3::new(0.0, 0.0, CURSOR_Z)),
            Anchor::BOTTOM_LEFT,
            Visibility::Hidden,
            CursorOverlay,
        ))
        .id();

    commands.insert_resource(TerminalEntities { cells, cursor });
}

fn pump_transport_input(
    mut transport: ResMut<TransportResource>,
    mut terminal: ResMut<TerminalState>,
) {
    loop {
        match transport.read_available() {
            Ok(Some(bytes)) => {
                for byte in bytes {
                    if let Err(err) = terminal.parser.consume(byte) {
                        error!("failed to consume PTY byte {byte:#04X}: {err}");
                    }
                }
            }
            Ok(None) => break,
            Err(err) => {
                error!("failed to read emulator PTY: {err}");
                break;
            }
        }
    }
}

fn capture_keyboard_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut transport: ResMut<TransportResource>,
) {
    let mut bytes = Vec::new();

    for event in keyboard_inputs.read() {
        if !event.state.is_pressed() {
            continue;
        }

        if let Some(text) = &event.text {
            bytes.extend(text.chars().filter_map(map_character));
            continue;
        }

        if let Some(mapped) = map_named_key(event.key_code) {
            bytes.push(mapped);
        }
    }

    if !bytes.is_empty() {
        if let Err(err) = transport.write_all(&bytes) {
            error!("failed to write keyboard bytes to PTY: {err}");
        }
    }
}

fn advance_blink_phase(time: Res<Time>, mut terminal: ResMut<TerminalState>) {
    if terminal.blink_timer.tick(time.delta()).just_finished() {
        terminal.blink_visible = !terminal.blink_visible;
    }
}

fn render_terminal(
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
            let width = if owner_cell.attributes.double_width {
                2.0
            } else {
                1.0
            };
            let height = if owner_cell.attributes.double_height {
                2.0
            } else {
                1.0
            };
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

fn map_named_key(code: KeyCode) -> Option<u8> {
    match code {
        KeyCode::Enter | KeyCode::NumpadEnter => Some(b'\r'),
        KeyCode::Backspace => Some(0x08),
        KeyCode::Tab => Some(b'\t'),
        KeyCode::Space => Some(b' '),
        _ => None,
    }
}

fn cell_index(x: u8, y: u8) -> usize {
    ((y - 1) as usize * SCREEN_COLUMNS as usize) + (x - 1) as usize
}

fn cell_position(config: &EmulatorConfig, x: u8, y: u8) -> Vec2 {
    let screen = config.screen_size();
    let left = -screen.x / 2.0;
    let top = screen.y / 2.0;

    Vec2::new(
        left + (x - 1) as f32 * config.cell_size.x,
        top - y as f32 * config.cell_size.y,
    )
}

fn palette_for(cell: &Cell, colors: bool) -> (Color, Color) {
    let mut foreground = palette_color(cell.attributes.foreground, colors);
    let mut background = palette_color(cell.attributes.background, colors);

    if cell.attributes.invert {
        std::mem::swap(&mut foreground, &mut background);
    }

    (foreground, background)
}

fn palette_color(index: u8, colors: bool) -> Color {
    if colors {
        match index {
            0 => Color::srgb_u8(16, 16, 16),
            1 => Color::srgb_u8(222, 56, 43),
            2 => Color::srgb_u8(57, 181, 74),
            3 => Color::srgb_u8(255, 199, 6),
            4 => Color::srgb_u8(0, 111, 184),
            5 => Color::srgb_u8(118, 38, 113),
            6 => Color::srgb_u8(44, 181, 233),
            _ => Color::srgb_u8(240, 240, 240),
        }
    } else {
        match index {
            0 => Color::srgb_u8(16, 16, 16),
            1 => Color::srgb_u8(76, 76, 76),
            2 => Color::srgb_u8(102, 102, 102),
            3 => Color::srgb_u8(127, 127, 127),
            4 => Color::srgb_u8(153, 153, 153),
            5 => Color::srgb_u8(178, 178, 178),
            6 => Color::srgb_u8(204, 204, 204),
            _ => Color::srgb_u8(240, 240, 240),
        }
    }
}
