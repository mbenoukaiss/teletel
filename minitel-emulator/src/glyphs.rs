use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use font8x8::{UnicodeFonts, BASIC_FONTS};
use std::collections::HashMap;
use teletel_protocol::parser::{Cell, CharacterSet};

const GLYPH_WIDTH: usize = 8;
const GLYPH_HEIGHT: usize = 10;

#[derive(Resource, Default)]
pub struct GlyphCache {
    handles: HashMap<GlyphKey, Handle<Image>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GlyphKey {
    Text(char),
    SemiGraphic { code: u8, separated: bool },
}

impl GlyphCache {
    pub fn glyph_for(&mut self, images: &mut Assets<Image>, cell: &Cell) -> Option<Handle<Image>> {
        if cell.content == '\0' || cell.content == ' ' {
            return None;
        }

        let key = match cell.attributes.character_set {
            CharacterSet::G0 => GlyphKey::Text(normalize_text_glyph(cell.content)),
            CharacterSet::G1 => GlyphKey::SemiGraphic {
                code: cell.content as u8,
                separated: cell.attributes.underline,
            },
        };

        if let Some(handle) = self.handles.get(&key) {
            return Some(handle.clone());
        }

        let image = match key {
            GlyphKey::Text(character) => build_text_glyph(character),
            GlyphKey::SemiGraphic { code, separated } => build_semigraphic_glyph(code, separated),
        };

        let handle = images.add(image);
        self.handles.insert(key, handle.clone());
        Some(handle)
    }
}

fn normalize_text_glyph(character: char) -> char {
    match character {
        '\u{7f}' => '\u{2588}',
        '\0' => ' ',
        other => other,
    }
}

fn build_text_glyph(character: char) -> Image {
    let bitmap = glyph_bitmap(character)
        .or_else(|| glyph_bitmap('?'))
        .unwrap_or([0; 8]);
    let mut pixels = vec![0; GLYPH_WIDTH * GLYPH_HEIGHT * 4];

    for (row, byte) in bitmap.iter().enumerate() {
        let y = row + 1;
        for x in 0..GLYPH_WIDTH {
            let bit = (byte >> x) & 1;
            if bit == 1 {
                set_pixel(&mut pixels, GLYPH_WIDTH, x, y, [255, 255, 255, 255]);
            }
        }
    }

    new_image(pixels)
}

fn glyph_bitmap(character: char) -> Option<[u8; 8]> {
    BASIC_FONTS
        .get(character)
        .or_else(|| font8x8::LATIN_FONTS.get(character))
        .or_else(|| font8x8::MISC_FONTS.get(character))
}

fn build_semigraphic_glyph(code: u8, separated: bool) -> Image {
    let mut pixels = vec![0; GLYPH_WIDTH * GLYPH_HEIGHT * 4];
    let value = if code == 0x7f { 0x5f } else { code };
    let pattern = (value & 0x1F) | ((value & 0x40) >> 1);
    let x_ranges = [(0usize, 4usize), (4usize, 8usize)];
    let y_ranges = [(0usize, 3usize), (3usize, 6usize), (6usize, 10usize)];

    for row in 0..3 {
        for col in 0..2 {
            let bit_index = row * 2 + col;
            if (pattern >> bit_index) & 1 == 0 {
                continue;
            }

            let (mut x0, mut x1) = x_ranges[col];
            let (mut y0, mut y1) = y_ranges[row];

            if separated {
                if col == 0 {
                    x1 = x1.saturating_sub(1);
                } else {
                    x0 += 1;
                }

                if row == 0 {
                    y1 = y1.saturating_sub(1);
                } else if row == 1 {
                    y0 += 1;
                    y1 = y1.saturating_sub(1);
                } else {
                    y0 += 1;
                }
            }

            for y in y0..y1 {
                for x in x0..x1 {
                    set_pixel(&mut pixels, GLYPH_WIDTH, x, y, [255, 255, 255, 255]);
                }
            }
        }
    }

    new_image(pixels)
}

fn set_pixel(pixels: &mut [u8], width: usize, x: usize, y: usize, rgba: [u8; 4]) {
    let offset = (y * width + x) * 4;
    pixels[offset..offset + 4].copy_from_slice(&rgba);
}

fn new_image(data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width: GLYPH_WIDTH as u32,
            height: GLYPH_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
