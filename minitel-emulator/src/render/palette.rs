use crate::config::{EmulatorConfig, SCREEN_COLUMNS};
use bevy::prelude::*;
use teletel_protocol::parser::Cell;

pub(super) fn cell_index(x: u8, y: u8) -> usize {
    ((y - 1) as usize * SCREEN_COLUMNS as usize) + (x - 1) as usize
}

pub(super) fn cell_position(config: &EmulatorConfig, x: u8, y: u8) -> Vec2 {
    let screen = config.screen_size();
    let left = -screen.x / 2.0;
    let top = screen.y / 2.0;

    Vec2::new(
        left + (x - 1) as f32 * config.cell_size.x,
        top - y as f32 * config.cell_size.y,
    )
}

pub(super) fn palette_for(cell: &Cell, colors: bool) -> (Color, Color) {
    let mut foreground = palette_color(cell.attributes.foreground, colors);
    let mut background = palette_color(cell.attributes.background, colors);

    if cell.attributes.invert {
        std::mem::swap(&mut foreground, &mut background);
    }

    (foreground, background)
}

pub(super) fn palette_color(index: u8, colors: bool) -> Color {
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
