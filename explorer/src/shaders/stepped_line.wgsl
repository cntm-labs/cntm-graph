/**
 * Stepped Line Shader (WGSL)
 * Calculates 3-segment orthogonal paths directly on GPU.
 */

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) src_pos: vec2<f32>,
    @location(1) tgt_pos: vec2<f32>,
    @location(2) t: f32, // 0.0 to 1.0 along the path
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // Orthogonal Routing: Right -> Down -> Right
    // Midpoint X for the vertical drop
    let mid_x = (src_pos.x + tgt_pos.x) * 0.5;

    var pos: vec2<f32>;

    // Divide path into 3 segments:
    // 0.0 - 0.33: Horizontal to mid_x
    // 0.33 - 0.66: Vertical to tgt_pos.y
    // 0.66 - 1.0: Horizontal to tgt_pos.x
    if (t < 0.33) {
        let local_t = t / 0.33;
        pos = mix(src_pos, vec2<f32>(mid_x, src_pos.y), local_t);
    } else if (t < 0.66) {
        let local_t = (t - 0.33) / 0.33;
        pos = mix(vec2<f32>(mid_x, src_pos.y), vec2<f32>(mid_x, tgt_pos.y), local_t);
    } else {
        let local_t = (t - 0.66) / 0.34;
        pos = mix(vec2<f32>(mid_x, tgt_pos.y), tgt_pos, local_t);
    }

    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return neon color
    return in.color;
}
