use crate::config::{EmulatorConfig, GUIDE_PANEL_WIDTH, SCREEN_COLUMNS, SCREEN_ROWS};
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
const DEBUG_HIGHLIGHT_Z: f32 = 2.0;
const BACKGROUND_Z: f32 = 0.0;
const FOREGROUND_Z: f32 = 1.0;

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphCache>()
            .init_resource::<TerminalState>()
            .add_systems(Startup, (setup_terminal, setup_beep_sound))
            .add_systems(
                Update,
                (
                    pump_transport_input,
                    capture_keyboard_input,
                    advance_blink_phase,
                    render_terminal,
                    update_debug_overlay,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
struct BeepSound(Handle<AudioSource>);

const BEEP_SAMPLE_RATE: u32 = 44100;
const BEEP_FREQUENCY: u32 = 440;
const BEEP_DURATION_MS: u32 = 200;
const BEEP_NUM_SAMPLES: usize = (BEEP_SAMPLE_RATE * BEEP_DURATION_MS / 1000) as usize;
const BEEP_WAV_SIZE: usize = 44 + BEEP_NUM_SAMPLES * 2;

const fn build_beep_wav() -> [u8; BEEP_WAV_SIZE] {
    let mut wav = [0u8; BEEP_WAV_SIZE];
    let data_size = (BEEP_NUM_SAMPLES * 2) as u32;
    let file_size = 36 + data_size;

    // RIFF header
    wav[0] = b'R'; wav[1] = b'I'; wav[2] = b'F'; wav[3] = b'F';
    let fs = file_size.to_le_bytes();
    wav[4] = fs[0]; wav[5] = fs[1]; wav[6] = fs[2]; wav[7] = fs[3];
    wav[8] = b'W'; wav[9] = b'A'; wav[10] = b'V'; wav[11] = b'E';

    // fmt chunk
    wav[12] = b'f'; wav[13] = b'm'; wav[14] = b't'; wav[15] = b' ';
    wav[16] = 16; // chunk size (16 bytes)
    wav[20] = 1; // PCM format
    wav[22] = 1; // mono
    let sr = BEEP_SAMPLE_RATE.to_le_bytes();
    wav[24] = sr[0]; wav[25] = sr[1]; wav[26] = sr[2]; wav[27] = sr[3];
    let br = (BEEP_SAMPLE_RATE * 2).to_le_bytes(); // byte rate
    wav[28] = br[0]; wav[29] = br[1]; wav[30] = br[2]; wav[31] = br[3];
    wav[32] = 2; // block align
    wav[34] = 16; // bits per sample

    // data chunk
    wav[36] = b'd'; wav[37] = b'a'; wav[38] = b't'; wav[39] = b'a';
    let ds = data_size.to_le_bytes();
    wav[40] = ds[0]; wav[41] = ds[1]; wav[42] = ds[2]; wav[43] = ds[3];

    // Square wave samples at ~30% amplitude
    let half_period = BEEP_SAMPLE_RATE / BEEP_FREQUENCY / 2;
    let amplitude: i16 = 9830; // ~30% of i16::MAX
    let mut i = 0;
    while i < BEEP_NUM_SAMPLES {
        let phase = (i as u32 % (half_period * 2)) < half_period;
        let sample = if phase { amplitude } else { -amplitude };
        let s = sample.to_le_bytes();
        wav[44 + i * 2] = s[0];
        wav[44 + i * 2 + 1] = s[1];
        i += 1;
    }

    wav
}

static BEEP_WAV: [u8; BEEP_WAV_SIZE] = build_beep_wav();

fn setup_beep_sound(mut commands: Commands, mut audio_assets: ResMut<Assets<AudioSource>>) {
    let source = AudioSource { bytes: BEEP_WAV[..].into() };
    let handle = audio_assets.add(source);
    commands.insert_resource(BeepSound(handle));
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

#[derive(Component)]
struct DebugHighlight;

#[derive(Component)]
struct DebugMouseCol;

#[derive(Component)]
struct DebugMouseRow;

#[derive(Component)]
struct DebugCursorCol;

#[derive(Component)]
struct DebugCursorRow;

fn setup_terminal(mut commands: Commands, config: Res<EmulatorConfig>) {
    let screen_size = config.screen_size();
    let total_height = screen_size.y + config.cell_size.y * 2.0;

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: total_height,
            },
            ..OrthographicProjection::default_2d()
        }),
        // Shift camera right so the terminal stays visually centered-left
        // and the guide panel occupies the right side
        Transform::from_translation(Vec3::new(GUIDE_PANEL_WIDTH / 2.0, 0.0, 0.0)),
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

    // Key guide panel on the right side of the terminal
    let guide_x = screen_size.x / 2.0 + config.cell_size.x + 10.0;
    let guide_top = screen_size.y / 2.0 - config.cell_size.y;

    let title_style = (
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
    );

    let key_style = (
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
    );

    let entries: &[(&str, &str)] = &[
        ("F1", "Envoi"),
        ("F2", "Retour"),
        ("F3", "Repetition"),
        ("F4", "Guide"),
        ("F5", "Annulation"),
        ("F6", "Sommaire"),
        ("F7", "Suite"),
        ("F8", "Connexion/Fin"),
        ("F9", "Correction"),
        ("", ""),
        ("Arrows", "Cursor"),
        ("Bksp", "Correction"),
        ("Enter", "Envoi"),
        ("Esc", "Escape"),
    ];

    let label_offset = 70.0;

    commands.spawn((
        Text2d::new("Keyboard"),
        title_style.clone(),
        Anchor::TOP_LEFT,
        Transform::from_translation(Vec3::new(guide_x, guide_top, FOREGROUND_Z)),
    ));

    for (i, (key, label)) in entries.iter().enumerate() {
        let y = guide_top - 22.0 - i as f32 * 16.0;
        if !key.is_empty() {
            commands.spawn((
                Text2d::new(*key),
                key_style.clone(),
                Anchor::TOP_LEFT,
                Transform::from_translation(Vec3::new(guide_x, y, FOREGROUND_Z)),
            ));
            commands.spawn((
                Text2d::new(*label),
                key_style.clone(),
                Anchor::TOP_LEFT,
                Transform::from_translation(Vec3::new(guide_x + label_offset, y, FOREGROUND_Z)),
            ));
        }
    }

    // Debug overlay entities
    commands.spawn((
        Sprite::from_color(Color::srgba(0.5, 0.5, 0.5, 0.4), config.cell_size),
        Transform::from_translation(Vec3::new(0.0, 0.0, DEBUG_HIGHLIGHT_Z)),
        Anchor::BOTTOM_LEFT,
        Visibility::Hidden,
        DebugHighlight,
    ));

    // Info table: 3 columns (label, Col, Row) x 3 rows (header, Mouse, Cursor)
    let table_x = guide_x;
    let table_bottom = -screen_size.y / 2.0;
    let row_height = 16.0;
    let col1_x = table_x;
    let col2_x = table_x + 55.0;
    let col3_x = table_x + 95.0;

    // Header row
    let y = table_bottom + row_height * 2.0;
    for (x, text) in [(col2_x, "Col"), (col3_x, "Row")] {
        commands.spawn((
            Text2d::new(text),
            key_style.clone(),
            Anchor::BOTTOM_LEFT,
            Transform::from_translation(Vec3::new(x, y, FOREGROUND_Z)),
        ));
    }

    // Mouse row
    let y = table_bottom + row_height;
    commands.spawn((
        Text2d::new("Mouse"),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col1_x, y, FOREGROUND_Z)),
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col2_x, y, FOREGROUND_Z)),
        DebugMouseCol,
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col3_x, y, FOREGROUND_Z)),
        DebugMouseRow,
    ));

    // Cursor row
    let y = table_bottom;
    commands.spawn((
        Text2d::new("Cursor"),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col1_x, y, FOREGROUND_Z)),
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col2_x, y, FOREGROUND_Z)),
        DebugCursorCol,
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col3_x, y, FOREGROUND_Z)),
        DebugCursorRow,
    ));
}

fn pump_transport_input(
    mut commands: Commands,
    mut transport: ResMut<TransportResource>,
    mut terminal: ResMut<TerminalState>,
    beep_sound: Res<BeepSound>,
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

    // Send protocol response bytes back to the remote host
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

fn capture_keyboard_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut transport: ResMut<TransportResource>,
) {
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

fn update_debug_overlay(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    config: Res<EmulatorConfig>,
    terminal: Res<TerminalState>,
    mut highlight: Query<(&mut Transform, &mut Visibility), With<DebugHighlight>>,
    mut mouse_col: Query<&mut Text2d, (With<DebugMouseCol>, Without<DebugMouseRow>, Without<DebugCursorCol>, Without<DebugCursorRow>)>,
    mut mouse_row: Query<&mut Text2d, (With<DebugMouseRow>, Without<DebugMouseCol>, Without<DebugCursorCol>, Without<DebugCursorRow>)>,
    mut cursor_col: Query<&mut Text2d, (With<DebugCursorCol>, Without<DebugCursorRow>, Without<DebugMouseCol>, Without<DebugMouseRow>)>,
    mut cursor_row: Query<&mut Text2d, (With<DebugCursorRow>, Without<DebugCursorCol>, Without<DebugMouseCol>, Without<DebugMouseRow>)>,
) {
    let ctx = terminal.parser.ctx();

    if let Ok(mut text) = cursor_col.single_mut() {
        **text = format!("{}", ctx.cursor_x);
    }
    if let Ok(mut text) = cursor_row.single_mut() {
        **text = format!("{}", ctx.cursor_y);
    }

    let (Ok(window), Ok((camera, camera_transform))) =
        (window.single(), camera.single()) else { return };

    let Some(cursor_pos) = window.cursor_position() else {
        if let Ok((_, mut vis)) = highlight.single_mut() {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut text) = mouse_col.single_mut() {
            **text = String::new();
        }
        if let Ok(mut text) = mouse_row.single_mut() {
            **text = String::new();
        }
        return;
    };

    let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let screen = config.screen_size();
    let left = -screen.x / 2.0;
    let top = screen.y / 2.0;

    let col = ((world.x - left) / config.cell_size.x).floor() as i32 + 1;
    let row = ((top - world.y) / config.cell_size.y).floor() as i32 + 1;

    if col >= 1 && col <= SCREEN_COLUMNS as i32 && row >= 1 && row <= SCREEN_ROWS as i32 {
        let origin = cell_position(&config, col as u8, row as u8);

        if let Ok((mut transform, mut vis)) = highlight.single_mut() {
            transform.translation = origin.extend(DEBUG_HIGHLIGHT_Z);
            *vis = Visibility::Visible;
        }

        if let Ok(mut text) = mouse_col.single_mut() {
            **text = format!("{col}");
        }
        if let Ok(mut text) = mouse_row.single_mut() {
            **text = format!("{row}");
        }
    } else {
        if let Ok((_, mut vis)) = highlight.single_mut() {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut text) = mouse_col.single_mut() {
            **text = String::new();
        }
        if let Ok(mut text) = mouse_row.single_mut() {
            **text = String::new();
        }
    }
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
