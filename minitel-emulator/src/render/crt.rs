use crate::config::EmulatorConfig;
use bevy::camera::ScalingMode;
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{
    InternedRenderLabel, InternedRenderSubGraph, RenderLabel, RenderSubGraph,
};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

/// CRT post-processing material. `screen_bounds` stores the UV rect of the
/// terminal area (min_x, min_y, max_x, max_y) so the effect is confined to
/// the Minitel screen and does not touch the guide panel.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct CrtMaterial {
    /// Set to 1.0 to enable CRT effects, 0.0 to pass through unchanged.
    pub enabled: f32,
    pub scanline_intensity: f32,
    pub curvature: f32,
    pub vignette_intensity: f32,
    /// UV rect of the terminal: (min_x, min_y, max_x, max_y)
    pub screen_bounds: Vec4,
}

impl Default for CrtMaterial {
    fn default() -> Self {
        Self {
            enabled: 1.0,
            scanline_intensity: 0.3,
            curvature: 0.02,
            vignette_intensity: 0.25,
            screen_bounds: Vec4::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

impl FullscreenMaterial for CrtMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/crt.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }
}

/// Recomputes the UV bounds of the terminal screen area each frame so that the
/// CRT effect adapts if the window is resized.
pub(super) fn update_crt_screen_bounds(
    mut query: Query<(&mut CrtMaterial, &Transform, &Projection)>,
    windows: Query<&Window>,
    config: Res<EmulatorConfig>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut crt, transform, projection)) = query.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let ScalingMode::FixedVertical { viewport_height } = ortho.scaling_mode else {
        return;
    };

    let aspect = window.width() / window.height();
    let visible_width = viewport_height * aspect;
    let visible_height = viewport_height;

    let cam_x = transform.translation.x;
    let cam_y = transform.translation.y;

    let world_left = cam_x - visible_width / 2.0;
    let world_top = cam_y + visible_height / 2.0;

    let screen_size = config.screen_size();
    let (screen_left, screen_right, screen_top, screen_bottom) = crt_world_bounds(&screen_size);

    let bounds = Vec4::new(
        (screen_left - world_left) / visible_width,
        (world_top - screen_top) / visible_height,
        (screen_right - world_left) / visible_width,
        (world_top - screen_bottom) / visible_height,
    );

    if crt.screen_bounds != bounds {
        crt.screen_bounds = bounds;
    }
}

/// Small margin so boundary sprites (grid lines at terminal edges) fall inside
/// the CRT zone and get properly barrel-distorted.
const CRT_MARGIN: f32 = 2.0;

/// Returns the CRT world-space bounds (left, right, top, bottom) expanded by
/// [`CRT_MARGIN`].
pub(super) fn crt_world_bounds(screen_size: &Vec2) -> (f32, f32, f32, f32) {
    (
        -screen_size.x / 2.0 - CRT_MARGIN,
        screen_size.x / 2.0 + CRT_MARGIN,
        screen_size.y / 2.0 + CRT_MARGIN,
        -screen_size.y / 2.0 - CRT_MARGIN,
    )
}
