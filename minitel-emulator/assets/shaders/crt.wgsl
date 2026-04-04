#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtMaterial {
    enabled: f32,
    scanline_intensity: f32,
    curvature: f32,
    vignette_intensity: f32,
    // UV rect of the terminal area: (min_x, min_y, max_x, max_y)
    screen_bounds: vec4<f32>,
}

@group(0) @binding(2) var<uniform> settings: CrtMaterial;

// Barrel distortion: curves the UV coordinates to simulate CRT screen curvature.
// Operates in terminal-local coordinates (0..1 within the terminal rect).
fn barrel_distortion(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - vec2<f32>(0.5);
    let dist_sq = dot(centered, centered);
    let distorted = centered * (1.0 + amount * dist_sq);
    return distorted + vec2<f32>(0.5);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // When disabled, pass through unchanged
    if settings.enabled < 0.5 {
        return textureSample(screen_texture, texture_sampler, uv);
    }

    let bounds_min = settings.screen_bounds.xy;
    let bounds_max = settings.screen_bounds.zw;
    let bounds_size = bounds_max - bounds_min;

    // Check if this pixel is inside the terminal area
    let inside = uv.x >= bounds_min.x && uv.x <= bounds_max.x
              && uv.y >= bounds_min.y && uv.y <= bounds_max.y;

    if !inside {
        // Outside the terminal: pass through unchanged
        return textureSample(screen_texture, texture_sampler, uv);
    }

    // Map to terminal-local UV (0..1 within the terminal rect)
    let local_uv = (uv - bounds_min) / bounds_size;

    // Apply barrel distortion in local space
    let curved_local = barrel_distortion(local_uv, settings.curvature * 10.0);

    // If the distorted coordinate falls outside the terminal, show black
    if curved_local.x < 0.0 || curved_local.x > 1.0 || curved_local.y < 0.0 || curved_local.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Map back to full-screen UV for texture sampling
    let curved_uv = bounds_min + curved_local * bounds_size;

    // Chromatic aberration: slight RGB channel offset in screen-space
    let aberration = 0.001;
    let r = textureSample(screen_texture, texture_sampler, curved_uv + vec2<f32>(aberration, 0.0)).r;
    let g = textureSample(screen_texture, texture_sampler, curved_uv).g;
    let b = textureSample(screen_texture, texture_sampler, curved_uv - vec2<f32>(aberration, 0.0)).b;
    var color = vec3<f32>(r, g, b);

    // Scanlines: darken alternating rows based on terminal-local Y
    let screen_height = f32(textureDimensions(screen_texture).y);
    let terminal_pixel_y = curved_local.y * bounds_size.y * screen_height;
    let scanline = sin(terminal_pixel_y * 3.14159265) * 0.5 + 0.5;
    let scanline_factor = 1.0 - settings.scanline_intensity * (1.0 - scanline);
    color = color * scanline_factor;

    // Vignette: darken edges relative to the terminal center
    let centered = curved_local - vec2<f32>(0.5);
    let vignette = 1.0 - dot(centered, centered) * settings.vignette_intensity * 4.0;
    let vignette_clamped = clamp(vignette, 0.0, 1.0);
    color = color * vignette_clamped;

    // Slight brightness boost to compensate for scanline darkening
    color = color * (1.0 + settings.scanline_intensity * 0.2);

    return vec4<f32>(color, 1.0);
}
