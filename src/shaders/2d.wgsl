const KIND_RECT:     u32 = 0u;
const KIND_ELLIPSE:  u32 = 1u;
const KIND_LINE:     u32 = 2u;
const KIND_TEXTURED: u32 = 3u;
const KIND_SDF_TEXT: u32 = 4u; // TODO: this is unused for now until it's actually implemented

struct Vertex {
    @location(0) position:      vec2<f32>,
    @location(1) uv:            vec2<f32>,
    @location(2) radii:         vec2<f32>,
    @location(3) fill_color:    vec4<f32>,
    @location(4) outline_color: vec4<f32>,
    @location(5) outline_width: f32,
    @location(6) corner_radius: f32,
    @location(7) kind:          u32,
}

struct Interpolated {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv:            vec2<f32>,
    @location(1) radii:         vec2<f32>,
    @location(2) fill_color:    vec4<f32>,
    @location(3) outline_color: vec4<f32>,
    @location(4) outline_width: f32,
    @location(5) corner_radius: f32,
    @location(6) kind:          u32,
}

@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(in: Vertex) -> Interpolated {
    var out: Interpolated;
    out.clip_position = projection * vec4<f32>(in.position, 0.0, 1.0);
    out.uv            = in.uv;
    out.radii         = in.radii;
    out.fill_color    = in.fill_color;
    out.outline_color = in.outline_color;
    out.outline_width = in.outline_width;
    out.corner_radius = in.corner_radius;
    out.kind          = in.kind;
    return out;
}

// TODO: overlapping transparent shapes can look a bit bad with srgb blending
//       maybe a second pass with the shader that does gamma correction after all is rendered?
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / 2.2));
}

// TODO: the ability to choose where the outline lands on the shape
//       (inside/centered/outside)
fn apply_outline(
    dist: f32,
    fill: vec4<f32>,
    outline: vec4<f32>,
    outline_width: f32,
    aa: f32,
) -> vec4<f32> {
    let half_width = outline_width * 0.5;

    let inner_dist = dist - half_width;
    let outer_dist = dist + half_width;

    let outer_alpha = smoothstep(-aa, aa, -inner_dist);
    let fill_factor = smoothstep(-aa, aa, -outer_dist);

    let color = mix(outline, fill, fill_factor);
    let a = color.a * outer_alpha;

    let srgb_color = linear_to_srgb(color.rgb);

    return vec4(srgb_color * a, a);
}

fn sdf_box(uv: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let d = abs(uv) - half_size + r;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

@fragment
fn fs_main(in: Interpolated) -> @location(0) vec4<f32> {
    // TODO: AA toggling
    let aa = 0.7071; // approx. sqrt(2)/2
    // let aa = 0.;

    if in.kind == KIND_ELLIPSE {
        let p = in.uv / in.radii;
        let l = length(p);

        let grad = p / in.radii;
        let grad_len = length(grad);

        let dist = (l - 1.0) / max(grad_len, 1e-5);

        return apply_outline(dist, in.fill_color, in.outline_color, in.outline_width, aa);
    }

    if in.kind == KIND_LINE {
        let nearest = vec2(clamp(in.uv.x, -in.radii.x, in.radii.x), 0.0);

        let dist = length(in.uv - nearest);

        return apply_outline(dist, in.fill_color, in.outline_color, in.outline_width, aa);
    }

    if in.kind == KIND_TEXTURED {
        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv) * in.fill_color;
        let srgb = linear_to_srgb(tex_color.rgb);
        return vec4<f32>(srgb * tex_color.a, tex_color.a);
    }

    // KIND_RECT
    let dist = sdf_box(in.uv, in.radii, in.corner_radius);
    return apply_outline(dist, in.fill_color, in.outline_color, in.outline_width, aa);
}
