@group(2) @binding(0) var<uniform> color: vec4<f32>;

struct FragmentOutput {
    @builtin(frag_depth) frag_depth: f32,
    @location(0) color: vec4<f32>,
}

@fragment
fn fragment() -> FragmentOutput {
    var out: FragmentOutput;
    out.frag_depth = -1.0;
    out.color = color;
    return out;
}
