use bevy::prelude::*;
use bevy::window::WindowResolution;

pub const SCREEN_COLUMNS: u8 = 40;
pub const SCREEN_ROWS: u8 = 24;

#[derive(Resource, Clone, Debug)]
pub struct EmulatorConfig {
    pub title: String,
    pub cell_size: Vec2,
    pub window_resolution: WindowResolution,
    pub colors: bool,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        let cell_size = Vec2::new(16.0, 20.0);
        let screen_size = Vec2::new(
            SCREEN_COLUMNS as f32 * cell_size.x,
            SCREEN_ROWS as f32 * cell_size.y,
        );
        let padding = Vec2::new(48.0, 48.0);
        let resolution = screen_size + padding;

        Self {
            title: "Minitel Emulator".to_owned(),
            cell_size,
            window_resolution: WindowResolution::new(
                resolution.x.round() as u32,
                resolution.y.round() as u32,
            ),
            colors: false,
        }
    }
}

impl EmulatorConfig {
    pub fn screen_size(&self) -> Vec2 {
        Vec2::new(
            SCREEN_COLUMNS as f32 * self.cell_size.x,
            SCREEN_ROWS as f32 * self.cell_size.y,
        )
    }
}
