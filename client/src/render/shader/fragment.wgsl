const PI: f32 = 3.14159265359;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec2<f32>,

    @location(2) @interpolate(flat) shape_type: u32,
    @location(3) @interpolate(flat) sides: u32,
    @location(4) @interpolate(flat) fill_color: vec4<f32>,
    @location(5) @interpolate(flat) border_color: vec4<f32>,
    @location(6) @interpolate(flat) border_thickness: f32,
    @location(7) @interpolate(flat) extra_param: f32,
};

fn draw_grid(
    world_pos: vec2<f32>,
    cell_size: f32,
    line_width: f32,
    bg_color: vec4<f32>,
    line_color: vec4<f32>,
    line_aa: f32
) -> vec4<f32> {
    let grid_coord = abs(fract(world_pos / cell_size - 0.5) - 0.5) * cell_size;
    let dist_to_line = min(grid_coord.x, grid_coord.y);

    let half_width = line_width * 0.5;
    let line_factor = 1.0 - smoothstep(half_width - line_aa, half_width + line_aa, dist_to_line);
    return mix(bg_color, line_color, line_factor);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_fwidth = fwidth(in.world_pos);
    let grid_line_aa = max(world_fwidth.x, world_fwidth.y);

    let uv_fwidth = fwidth(in.uv);
    let delta = max(uv_fwidth.x, uv_fwidth.y);

    if (in.shape_type == 2u) {
        let cell_size = in.extra_param;
        return draw_grid(
            in.world_pos,
            cell_size,
            in.border_thickness,
            in.fill_color,
            in.border_color,
            grid_line_aa,
        );
    }

    if (in.shape_type == 0u) {
        let dist_circle = length(in.uv);
        let alpha = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, dist_circle);
        if (alpha < 0.001) {
            discard;
        }

        let border_uv_width = in.border_thickness * delta;
        let border_mix = smoothstep(1.0 - border_uv_width - delta, 1.0 - border_uv_width + delta, dist_circle);

        let final_color = mix(in.fill_color, in.border_color, border_mix);
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    }

    if (in.shape_type == 1u) {
        let dist_box = max(abs(in.uv.x), abs(in.uv.y));
        let alpha = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, dist_box);
        if (alpha < 0.001) {
            discard;
        }

        let border_uv_width = in.border_thickness * delta;
        let border_mix = smoothstep(1.0 - border_uv_width - delta, 1.0 - border_uv_width + delta, dist_box);

        let final_color = mix(in.fill_color, in.border_color, border_mix);
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    }

    if (in.shape_type == 3u && in.sides >= 3u) {
        let sides_f = f32(in.sides);
        let angle = atan2(in.uv.y, in.uv.x);
        let slice = (2.0 * PI) / sides_f;

        let apothem = cos(PI / sides_f);
        let dist_poly = (cos(floor(0.5 + angle / slice) * slice - angle) * length(in.uv)) / apothem;

        let alpha = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, dist_poly);
        if (alpha < 0.001) {
            discard;
        }

        let border_uv_width = in.border_thickness * delta;
        let border_mix = smoothstep(1.0 - border_uv_width - delta, 1.0 - border_uv_width + delta, dist_poly);

        let final_color = mix(in.fill_color, in.border_color, border_mix);
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    }

    return in.fill_color;
}
