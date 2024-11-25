[[group(0), binding(0)]] var skybox_texture: texture_cube<f32>;
[[group(0), binding(1)]] var skybox_sampler: sampler;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coords: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var positions = array<vec3<f32>, 36>(
        vec3(-1.0, -1.0, -1.0), vec3(1.0, -1.0, -1.0), vec3(1.0,  1.0, -1.0),
        vec3(-1.0, -1.0, -1.0), vec3(1.0,  1.0, -1.0), vec3(-1.0,  1.0, -1.0),
        vec3(-1.0, -1.0,  1.0), vec3(1.0, -1.0,  1.0), vec3(1.0,  1.0,  1.0),
        vec3(-1.0, -1.0,  1.0), vec3(1.0,  1.0,  1.0), vec3(-1.0,  1.0,  1.0),
        vec3(-1.0, -1.0, -1.0), vec3(-1.0, -1.0,  1.0), vec3(-1.0,  1.0,  1.0),
        vec3(-1.0, -1.0, -1.0), vec3(-1.0,  1.0,  1.0), vec3(-1.0,  1.0, -1.0),
        vec3(1.0, -1.0, -1.0), vec3(1.0, -1.0,  1.0), vec3(1.0,  1.0,  1.0),
        vec3(1.0, -1.0, -1.0), vec3(1.0,  1.0,  1.0), vec3(1.0,  1.0, -1.0),
        vec3(-1.0, -1.0, -1.0), vec3(-1.0, -1.0,  1.0), vec3(1.0, -1.0,  1.0),
        vec3(-1.0, -1.0, -1.0), vec3(1.0, -1.0,  1.0), vec3(1.0, -1.0, -1.0),
        vec3(-1.0,  1.0, -1.0), vec3(-1.0,  1.0,  1.0), vec3(1.0,  1.0,  1.0),
        vec3(-1.0,  1.0, -1.0), vec3(1.0,  1.0,  1.0), vec3(1.0,  1.0, -1.0)
    );
    let position = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    output.tex_coords = position;
    return output;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(skybox_texture, skybox_sampler, input.tex_coords);
}
