use crate::config::{EmulatorConfig, SCREEN_COLUMNS, SCREEN_ROWS};
use bevy::prelude::*;

use super::palette::cell_position;
use super::TerminalState;

// --- Types ---

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum BaudRate {
    Unlimited,
    B300,
    B1200,
    B4800,
    B9600,
}

impl BaudRate {
    pub fn next(self) -> Self {
        if self == BaudRate::Unlimited { BaudRate::B300 }
        else if self == BaudRate::B300 { BaudRate::B1200 }
        else if self == BaudRate::B1200 { BaudRate::B4800 }
        else if self == BaudRate::B4800 { BaudRate::B9600 }
        else { BaudRate::Unlimited }
    }

    pub fn bytes_per_second(self) -> Option<f32> {
        if self == BaudRate::B300 { Some(30.0) }
        else if self == BaudRate::B1200 { Some(120.0) }
        else if self == BaudRate::B4800 { Some(480.0) }
        else if self == BaudRate::B9600 { Some(960.0) }
        else { None }
    }

    pub fn label(self) -> &'static str {
        if self == BaudRate::Unlimited { "Unlim." }
        else if self == BaudRate::B300 { "300" }
        else if self == BaudRate::B1200 { "1200" }
        else if self == BaudRate::B4800 { "4800" }
        else { "9600" }
    }
}

#[derive(Resource)]
pub(super) struct DebugState {
    pub grid_visible: bool,
    pub cursor_highlight: bool,
    pub mouse_highlight: bool,
    pub baud_rate: BaudRate,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            grid_visible: false,
            cursor_highlight: false,
            mouse_highlight: true,
            baud_rate: BaudRate::Unlimited,
        }
    }
}

// --- Marker components ---

#[derive(Component)]
pub(super) struct DebugHighlight;

#[derive(Component)]
pub(super) struct DebugCursorHighlight;

#[derive(Component)]
pub(super) struct GridOverlay;

#[derive(Component)]
pub(super) struct DebugBaudText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum DebugToggleLabel {
    Grid,
    Cursor,
    Mouse,
}

#[derive(Component)]
pub(super) struct DebugMouseCol;

#[derive(Component)]
pub(super) struct DebugMouseRow;

#[derive(Component)]
pub(super) struct DebugCursorCol;

#[derive(Component)]
pub(super) struct DebugCursorRow;

// --- Systems ---

pub(super) fn handle_debug_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut baud_text: Query<&mut Text2d, With<DebugBaudText>>,
    mut toggle_labels: Query<(&DebugToggleLabel, &mut TextColor)>,
) {
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl_held {
        if keys.just_pressed(KeyCode::KeyG) {
            debug_state.grid_visible = !debug_state.grid_visible;
        }
        if keys.just_pressed(KeyCode::KeyC) {
            debug_state.cursor_highlight = !debug_state.cursor_highlight;
        }
        // KeyCode::Semicolon covers AZERTY where M is at the semicolon position
        if keys.just_pressed(KeyCode::KeyM) || keys.just_pressed(KeyCode::Semicolon) {
            debug_state.mouse_highlight = !debug_state.mouse_highlight;
        }
        if keys.just_pressed(KeyCode::KeyB) {
            debug_state.baud_rate = debug_state.baud_rate.next();
            if let Ok(mut text) = baud_text.single_mut() {
                **text = debug_state.baud_rate.label().to_string();
            }
        }
    }

    // always update toggle label colors
    let enabled_color = Color::srgb(0.8, 0.8, 0.8);
    let disabled_color = Color::srgb(0.5, 0.5, 0.5);

    for (toggle, mut color) in &mut toggle_labels {
        let active = if *toggle == DebugToggleLabel::Grid { debug_state.grid_visible }
        else if *toggle == DebugToggleLabel::Cursor { debug_state.cursor_highlight }
        else { debug_state.mouse_highlight };

        *color = TextColor(if active { enabled_color } else { disabled_color });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_debug_overlay(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    config: Res<EmulatorConfig>,
    terminal: Res<TerminalState>,
    debug_state: Res<DebugState>,
    mut highlight: Query<(&mut Transform, &mut Visibility), (With<DebugHighlight>, Without<DebugCursorHighlight>, Without<GridOverlay>)>,
    mut cursor_hl: Query<(&mut Transform, &mut Visibility), (With<DebugCursorHighlight>, Without<DebugHighlight>, Without<GridOverlay>)>,
    mut grid_lines: Query<&mut Visibility, (With<GridOverlay>, Without<DebugHighlight>, Without<DebugCursorHighlight>)>,
    mut mouse_col: Query<&mut Text2d, (With<DebugMouseCol>, Without<DebugMouseRow>, Without<DebugCursorCol>, Without<DebugCursorRow>)>,
    mut mouse_row: Query<&mut Text2d, (With<DebugMouseRow>, Without<DebugMouseCol>, Without<DebugCursorCol>, Without<DebugCursorRow>)>,
    mut cursor_col: Query<&mut Text2d, (With<DebugCursorCol>, Without<DebugCursorRow>, Without<DebugMouseCol>, Without<DebugMouseRow>)>,
    mut cursor_row: Query<&mut Text2d, (With<DebugCursorRow>, Without<DebugCursorCol>, Without<DebugMouseCol>, Without<DebugMouseRow>)>,
) {
    let ctx = terminal.parser.ctx();

    // cursor position text
    if let Ok(mut text) = cursor_col.single_mut() {
        **text = format!("{}", ctx.cursor_x);
    }
    if let Ok(mut text) = cursor_row.single_mut() {
        **text = format!("{}", ctx.cursor_y);
    }

    // grid overlay
    let grid_vis = if debug_state.grid_visible { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in &mut grid_lines {
        *vis = grid_vis;
    }

    // cursor highlight
    if debug_state.cursor_highlight {
        let origin = cell_position(&config, ctx.cursor_x, ctx.cursor_y);
        if let Ok((mut transform, mut vis)) = cursor_hl.single_mut() {
            transform.translation = origin.extend(super::DEBUG_HIGHLIGHT_Z);
            *vis = Visibility::Visible;
        }
    } else {
        if let Ok((_, mut vis)) = cursor_hl.single_mut() {
            *vis = Visibility::Hidden;
        }
    }

    // mouse highlight
    if !debug_state.mouse_highlight {
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
    let top_edge = screen.y / 2.0;

    let col = ((world.x - left) / config.cell_size.x).floor() as i32 + 1;
    let row = ((top_edge - world.y) / config.cell_size.y).floor() as i32 + 1;

    if col >= 1 && col <= SCREEN_COLUMNS as i32 && row >= 1 && row <= SCREEN_ROWS as i32 {
        let origin = cell_position(&config, col as u8, row as u8);

        if let Ok((mut transform, mut vis)) = highlight.single_mut() {
            transform.translation = origin.extend(super::DEBUG_HIGHLIGHT_Z);
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
