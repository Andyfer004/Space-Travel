@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Define positions for the triangle vertices
    var pos: vec2<f32>;
    switch (vertex_index) {
        case 0u: { pos = vec2<f32>(0.0, 0.5); }
        case 1u: { pos = vec2<f32>(-0.5, -0.5); }
        case 2u: { pos = vec2<f32>(0.5, -0.5); }
        default: { pos = vec2<f32>(0.0, 0.0); } // Fallback (not used but required)
    }

    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Output color (red)
}
