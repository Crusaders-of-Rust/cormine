#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> light_color: vec4<f32>;
@group(2) @binding(2) var<uniform> light_dir: vec3<f32>;

const AMBIENT_STRENGTH: f32 = 0.1;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let norm = normalize(mesh.world_normal);
    let light_dir = normalize(light_dir);
    let diff_strength = max(dot(norm, light_dir), 0.0);
    let diff_color = light_color * diff_strength;
    let ambient_color = light_color * AMBIENT_STRENGTH;

    return material_color * (ambient_color + diff_color);
}