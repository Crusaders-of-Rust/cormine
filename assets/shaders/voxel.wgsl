#import bevy_pbr::mesh_functions;
#import bevy_pbr::view_transformations;

@group(2) @binding(1) var<uniform> light_color: vec4<f32>;
@group(2) @binding(2) var<uniform> light_dir: vec3<f32>;
@group(2) @binding(3) var<uniform> selected_voxel: vec3<f32>;
@group(2) @binding(4) var<uniform> has_selected: u32;
@group(2) @binding(5) var texture: texture_2d_array<f32>;
@group(2) @binding(6) var texture_sampler: sampler;

const AMBIENT_STRENGTH: f32 = 0.1;
const OUTLINE_THICKNESS: f32 = 0.1;
// Y component of the vector from the player to the sun at which point
// the sun stops lighting things
const SUN_MIN_ANGLE: f32 = -0.3;
// Y component of the vector from the player to the sun at which point
// the sun is at its strongest (i.e. directly overhead)
const SUN_MAX_ANGLE: f32 = 1.0;

const SUN_MIN_STRENGTH: f32 = 0.0;

const SUN_MAX_STRENGTH: f32 = 1.0;


struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) vertex_data: u32,
}

var<private> VOXEL_NORMALS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(0., 0., -1.),
    vec3(0., 0., 1.),
    vec3(-1., 0., 0.),
    vec3(1., 0., 0.),
    vec3(0., -1., 0.),
    vec3(0., 1., 0.),
);

var<private> VOXEL_UVS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2(1.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
);

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) @interpolate(flat) vertex_data: u32,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) ao_level: f32,
}

// vertex_data bitfield
// N - Normal index
// M - Material
// U - UV index
// O - Number of neighbours (for AO)
// XXXXXXXX XXXXXXXX XXXXXXXO OUUMMMNNN

@vertex
fn vertex(vertex: Vertex) -> VertexOut {
    let model = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    let normal_idx = extractBits(vertex.vertex_data, 0u, 3u);

    var out: VertexOut;
    out.clip_position = view_transformations::position_world_to_clip(world_position.xyz);
    out.position = world_position.xyz;
    out.vertex_data = vertex.vertex_data;
    out.normal = VOXEL_NORMALS[normal_idx];
    out.uv = VOXEL_UVS[extractBits(vertex.vertex_data, 6u, 2u)];
    switch extractBits(vertex.vertex_data, 8u, 2u) {
        // Number of non-occluding neighbours of this vertex
        case 0u {
            out.ao_level = 0.1;
        }
        case 1u {
            out.ao_level = 0.25;
        }
        case 2u {
            out.ao_level= 0.5;
        }
        default {
            out.ao_level = 1.0;
        }
    }
    return out;
}

fn is_between(v: vec3<f32>, min: vec3<f32>, max: vec3<f32>) -> bool {
    // Fuzz this detection a bit to avoid z fighting
    let greater_than_min = all(v >= min - vec3(0.001));
    let less_than_max = all(v <= max + vec3(0.001));
    return greater_than_min && less_than_max;
}

fn map_range(value: f32, min_in: f32, max_in: f32, min_out: f32, max_out: f32) -> f32 {
    let factor = (value - min_in) / (max_in - min_in);
    return mix(min_out, max_out, factor);
}

@fragment
fn fragment(
    mesh: VertexOut,
) -> @location(0) vec4<f32> {
    let norm = normalize(mesh.normal);
    let light_dir = normalize(light_dir);
    // Strength of diffuse lighting according to angle between it and normal
    let diff_strength = max(dot(norm, light_dir), 0.0);
    // Strength of diffuse lighting based on sun direction - disabled below the horizon
    let diff_brightness = map_range(light_dir.y, SUN_MIN_ANGLE, SUN_MAX_ANGLE, SUN_MIN_STRENGTH, SUN_MAX_STRENGTH);

    let diff_color = light_color * diff_strength * diff_brightness;
    let ambient_color = light_color * AMBIENT_STRENGTH;
    let material_idx = extractBits(mesh.vertex_data, 3u, 3u);
    let material_color = textureSample(texture, texture_sampler, mesh.uv, material_idx);

#ifdef AO_DEBUG
    var out = vec4<f32>(mesh.ao_level);
#else
    var out = material_color * (ambient_color + diff_color);
    out *= mesh.ao_level;
#endif

    if bool(has_selected) && is_between(mesh.position, selected_voxel, selected_voxel + vec3(1.0)) {
        out *= 0.7;
    }
    return out;
}