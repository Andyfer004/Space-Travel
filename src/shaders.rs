pub const VERTEX_SHADER: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
};
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.view_proj * uniforms.model * vec4<f32>(position, 1.0);
    out.normal = normalize(position);
    out.color = uniforms.color;
    return out;
}
"#;

pub const FRAGMENT_SHADER_2: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.2;

    let rock_pattern = sin(normal.x * 10.0) * cos(normal.z * 10.0) +
                       sin(normal.y * 15.0) * cos(normal.x * 15.0);
    let rock_variation = smoothstep(-0.5, 0.5, rock_pattern);

    let base_rock_color = vec3<f32>(0.5, 0.4, 0.3);
    let highlight_color = vec3<f32>(0.7, 0.6, 0.5);
    let rock_color = mix(base_rock_color, highlight_color, rock_variation);

    return vec4<f32>((ambient + diffuse) * rock_color, color.a);
}
"#;

pub const FRAGMENT_SHADER_3: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.3;

    let gas_pattern = sin(normal.x * 5.0 + normal.y * 5.0) *
                      cos(normal.z * 8.0 + normal.x * 8.0);
    let density = smoothstep(-0.4, 0.4, gas_pattern);

    let gas_color = vec3<f32>(0.8, 0.6, 0.9);
    let highlight_color = vec3<f32>(1.0, 0.8, 1.0);
    let final_color = mix(gas_color, highlight_color, density);

    return vec4<f32>((ambient + diffuse) * final_color, color.a);
}
"#;

pub const FRAGMENT_SHADER_4: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let time = color.a;

    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.3;

    let noise = sin(normal.x * 8.0 + normal.y * 8.0 + normal.z * 8.0) *
                cos(normal.x * 10.0 + normal.y * 10.0);
    let terrain_pattern = smoothstep(-0.2, 0.2, noise);

    let land_color = vec3<f32>(0.2, 0.6, 0.2);
    let water_color = vec3<f32>(0.1, 0.3, 0.8);
    let base_surface = mix(water_color, land_color, terrain_pattern);

    let mountain_noise = sin(normal.x * 15.0 + normal.y * 15.0 + normal.z * 15.0) *
                         cos(normal.x * 20.0 + normal.z * 20.0);
    let mountain_pattern = smoothstep(0.4, 0.6, mountain_noise);
    let mountain_color = vec3<f32>(0.5, 0.4, 0.3);
    let surface_with_mountains = mix(base_surface, mountain_color, mountain_pattern);

    let cloud_pattern = sin((normal.x + time * 0.2) * 8.0) *
                        cos((normal.y + time * 0.2) * 8.0);
    let cloud_density = smoothstep(0.5, 0.7, cloud_pattern);
    let cloud_color = vec3<f32>(1.0, 1.0, 1.0);

    let final_surface = mix(surface_with_mountains, cloud_color, cloud_density);

    let final_color = (ambient + diffuse) * final_surface;

    return vec4<f32>(final_color, 1.0);
}
"#;

pub const FRAGMENT_SHADER_5: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.2;

    let strata_pattern = fract(normal.x * 10.0) * fract(normal.z * 15.0);
    let strata_variation = smoothstep(0.3, 0.7, strata_pattern);

    let base_strata_color = vec3<f32>(0.6, 0.5, 0.4);
    let lighter_strata_color = vec3<f32>(0.8, 0.7, 0.6);
    let strata_color = mix(base_strata_color, lighter_strata_color, strata_variation);

    return vec4<f32>((ambient + diffuse) * strata_color, color.a);
}
"#;

pub const FRAGMENT_SHADER_6: &str = r#"
@fragment
fn fs_main(
    @location(0) frag_position: vec3<f32>, 
    @location(1) _normal: vec3<f32>
) -> @location(0) vec4<f32> {
    let ring_distance = length(vec2<f32>(frag_position.x, frag_position.z));

    let inner_radius = 1.5;
    let outer_radius = 2.5;
    let ring_thickness = 0.2;

    let distance_color = clamp(ring_distance / outer_radius, 0.0, 1.0);
    let y_color = clamp(abs(frag_position.y) / ring_thickness, 0.0, 1.0);

    return vec4<f32>(distance_color, y_color, 0.0, 1.0);
}
"#;

pub const FRAGMENT_SHADER_7: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.3;

    let swirl_pattern = sin(normal.x * 12.0 + normal.y * 12.0) *
                        cos(normal.z * 15.0 + normal.x * 15.0);
    let swirl_density = smoothstep(-0.6, 0.6, swirl_pattern);

    let base_gas_color = vec3<f32>(0.7, 0.8, 1.0);
    let highlight_color = vec3<f32>(1.0, 1.0, 1.0);
    let swirl_color = mix(base_gas_color, highlight_color, swirl_density);

    return vec4<f32>((ambient + diffuse) * swirl_color, 1.0);
}
"#;

pub const FRAGMENT_SHADER_8: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(normal), light_dir), 0.0);
    let ambient = 0.2;

    let vignette = 1.0 - length(normal.xy);
    let rock_pattern = sin(normal.x * 8.0 + normal.z * 8.0) *
                       cos(normal.y * 10.0 + normal.z * 10.0);
    let rock_variation = smoothstep(-0.3, 0.3, rock_pattern);

    let base_rock_color = vec3<f32>(0.4, 0.3, 0.2);
    let highlighted_rock_color = vec3<f32>(0.7, 0.6, 0.5);
    let rock_color = mix(base_rock_color, highlighted_rock_color, rock_variation);

    return vec4<f32>((ambient + diffuse * vignette) * rock_color, color.a);
}
"#;

pub const FRAGMENT_SHADER_SUN: &str = r#"
@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let distance_from_center = length(normal.xy);
    
    // Núcleo central más intenso y brillante
    let core_intensity = 1.0 - smoothstep(0.0, 0.2, distance_from_center);
    let core_color = vec3<f32>(5.0, 4.5, 4.0) * core_intensity;
    
    // Corona más suave y gradual
    let corona_falloff = 1.0 - smoothstep(0.2, 1.0, distance_from_center);
    let corona_color = vec3<f32>(2.5, 1.8, 0.5) * corona_falloff;
    
    // Añadimos un brillo extra en el centro
    let bloom = pow(1.0 - distance_from_center, 4.0) * vec3<f32>(3.0, 2.5, 1.0);
    
    // Combinamos todos los efectos y aumentamos la intensidad general
    let final_color = (core_color + corona_color + bloom) * 2.0;
    
    return vec4<f32>(final_color, 1.0);
}
"#;
