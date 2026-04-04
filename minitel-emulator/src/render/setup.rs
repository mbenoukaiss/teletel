use crate::config::{EmulatorConfig, GUIDE_PANEL_WIDTH, SCREEN_COLUMNS, SCREEN_ROWS};
use bevy::camera::{OrthographicProjection, Projection, ScalingMode};
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::debug::*;
use super::palette::cell_position;
use super::{
    CellEntities, CursorOverlay, TerminalEntities, BACKGROUND_Z, CURSOR_Z, DEBUG_HIGHLIGHT_Z,
    FOREGROUND_Z,
};

pub(super) fn setup_terminal(mut commands: Commands, config: Res<EmulatorConfig>) {
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
        Transform::from_translation(Vec3::new(GUIDE_PANEL_WIDTH / 2.0, 0.0, 0.0)),
    ));

    // cell entities (background + foreground per cell)
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

    // --- Guide panel ---
    let guide_x = screen_size.x / 2.0 + config.cell_size.x + 10.0;
    let guide_top = screen_size.y / 2.0 - config.cell_size.y;
    let label_offset = 70.0;

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

    // keyboard section
    let keyboard_entries: &[(&str, &str)] = &[
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

    let mut y = guide_top;
    commands.spawn((
        Text2d::new("Keyboard"),
        title_style.clone(),
        Anchor::TOP_LEFT,
        Transform::from_translation(Vec3::new(guide_x, y, FOREGROUND_Z)),
    ));
    y -= 22.0;

    for (key, label) in keyboard_entries {
        if key.is_empty() {
            y -= 16.0;
            continue;
        }
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
        y -= 16.0;
    }

    // debug section
    y -= 8.0;
    commands.spawn((
        Text2d::new("Debug"),
        title_style.clone(),
        Anchor::TOP_LEFT,
        Transform::from_translation(Vec3::new(guide_x, y, FOREGROUND_Z)),
    ));
    y -= 22.0;

    let debug_entries: &[(&str, &str, Option<DebugToggleLabel>)] = &[
        ("Ctrl+G", "Grid", Some(DebugToggleLabel::Grid)),
        ("Ctrl+C", "Cursor", Some(DebugToggleLabel::Cursor)),
        ("Ctrl+M", "Mouse", Some(DebugToggleLabel::Mouse)),
        ("Ctrl+B", "Baud rate", None),
    ];

    let enabled_color = TextColor(Color::srgb(0.8, 0.8, 0.8));

    for (key, label, toggle) in debug_entries {
        commands.spawn((
            Text2d::new(*key),
            key_style.clone(),
            Anchor::TOP_LEFT,
            Transform::from_translation(Vec3::new(guide_x, y, FOREGROUND_Z)),
        ));
        let mut label_entity = commands.spawn((
            Text2d::new(*label),
            key_style.clone(),
            Anchor::TOP_LEFT,
            Transform::from_translation(Vec3::new(guide_x + label_offset, y, FOREGROUND_Z)),
        ));
        if let Some(toggle) = toggle {
            label_entity.insert(*toggle);
            if *toggle == DebugToggleLabel::Mouse {
                label_entity.insert(enabled_color.clone());
            }
        }
        y -= 16.0;
    }

    // --- Debug overlay sprites ---
    commands.spawn((
        Sprite::from_color(Color::srgba(0.5, 0.5, 0.5, 0.4), config.cell_size),
        Transform::from_translation(Vec3::new(0.0, 0.0, DEBUG_HIGHLIGHT_Z)),
        Anchor::BOTTOM_LEFT,
        Visibility::Hidden,
        DebugHighlight,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(0.5, 0.5, 0.5, 0.4), config.cell_size),
        Transform::from_translation(Vec3::new(0.0, 0.0, DEBUG_HIGHLIGHT_Z)),
        Anchor::BOTTOM_LEFT,
        Visibility::Hidden,
        DebugCursorHighlight,
    ));

    // grid overlay lines
    let grid_left = -screen_size.x / 2.0;
    let grid_top = screen_size.y / 2.0;
    let grid_bottom = grid_top - screen_size.y;
    let cw = config.cell_size.x;
    let ch = config.cell_size.y;
    let grid_strong = Color::srgba(1.0, 1.0, 1.0, 0.25);
    let grid_weak = Color::srgba(1.0, 1.0, 1.0, 0.05);

    for col in [0u8, 10, 20, 30, 40] {
        let color = if col == 0 || col == 20 || col == 40 { grid_strong } else { grid_weak };
        let x = grid_left + col as f32 * cw;
        commands.spawn((
            Sprite::from_color(color, Vec2::new(1.0, screen_size.y)),
            Transform::from_translation(Vec3::new(x, grid_bottom, DEBUG_HIGHLIGHT_Z)),
            Anchor::BOTTOM_LEFT,
            Visibility::Hidden,
            GridOverlay,
        ));
    }

    for row in [0u8, 6, 12, 18, 24] {
        let color = if row == 0 || row == 12 || row == 24 { grid_strong } else { grid_weak };
        let line_y = grid_top - row as f32 * ch;
        commands.spawn((
            Sprite::from_color(color, Vec2::new(screen_size.x, 1.0)),
            Transform::from_translation(Vec3::new(grid_left, line_y, DEBUG_HIGHLIGHT_Z)),
            Anchor::TOP_LEFT,
            Visibility::Hidden,
            GridOverlay,
        ));
    }

    // --- Info table ---
    let table_x = guide_x;
    let table_bottom = -screen_size.y / 2.0;
    let row_height = 16.0;
    let col1_x = table_x;
    let col2_x = table_x + 55.0;
    let col3_x = table_x + 95.0;

    let ty = table_bottom + row_height * 3.0 + 6.0;
    commands.spawn((
        Text2d::new("Information"),
        title_style,
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(table_x, ty, FOREGROUND_Z)),
    ));

    // mouse row
    let ty = table_bottom + row_height * 2.0;
    commands.spawn((
        Text2d::new("Mouse"),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col1_x, ty, FOREGROUND_Z)),
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col2_x, ty, FOREGROUND_Z)),
        DebugMouseCol,
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col3_x, ty, FOREGROUND_Z)),
        DebugMouseRow,
    ));

    // cursor row
    let ty = table_bottom + row_height;
    commands.spawn((
        Text2d::new("Cursor"),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col1_x, ty, FOREGROUND_Z)),
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col2_x, ty, FOREGROUND_Z)),
        DebugCursorCol,
    ));
    commands.spawn((
        Text2d::new(""),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col3_x, ty, FOREGROUND_Z)),
        DebugCursorRow,
    ));

    // baud row
    let ty = table_bottom;
    commands.spawn((
        Text2d::new("Baud"),
        key_style.clone(),
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col1_x, ty, FOREGROUND_Z)),
    ));
    commands.spawn((
        Text2d::new("Unlim."),
        key_style,
        Anchor::BOTTOM_LEFT,
        Transform::from_translation(Vec3::new(col2_x, ty, FOREGROUND_Z)),
        DebugBaudText,
    ));
}
