use crate::{KIND_CANVAS, KIND_ELLIPSE, KIND_LINE, KIND_RECT, KIND_TEXTURED, Vertex, types::Color, vec::Vec2};

#[allow(clippy::too_many_arguments)]
pub(crate) fn rect_vertices(
    x: f32, y: f32,
    w: f32, h: f32,
    fill_color: Color,
    outline_color: Color,
    outline_width: f32,
    corner_radius: f32,
) -> [Vertex; 6] {
    let (hw, hh) = (w / 2.0, h / 2.0);
    let (center_x, center_y) = (x + hw, y + hh);

    // half the outline width + an extra 1.5 for anti-aliasing
    let padding = (outline_width / 2.0) + 1.5;
    let pad_x = hw + padding;
    let pad_y = hh + padding;

    let v = |x, y| Vertex {
        position: Vec2::new(x + center_x, y + center_y),
        uv:       Vec2::new(x, y),
        radii:    Vec2::new(hw, hh),

        fill_color,
        outline_color,
        outline_width,

        corner_radius,

        kind: KIND_RECT,
    };
    [
        v(-pad_x, -pad_y), v( pad_x, -pad_y), v( pad_x,  pad_y), // triangle 1
        v(-pad_x, -pad_y), v( pad_x,  pad_y), v(-pad_x,  pad_y), // triangle 2
    ]
}

pub(crate) fn ellipse_vertices(
    x: f32, y: f32,
    rx: f32, ry: f32,
    fill_color: Color,
    outline_color: Color,
    outline_width: f32,
) -> [Vertex; 6] {
    // half the outline width + an extra 1.5 for anti-aliasing
    let padding = (outline_width / 2.0) + 1.5;
    let pad_x = rx + padding;
    let pad_y = ry + padding;

    let (x1, y1, x2, y2) = (x - pad_x, y - pad_y, x + pad_x, y + pad_y);

    let v = |x, y, ux, uy| Vertex {
        position: Vec2::new(x, y),
        uv:       Vec2::new(ux, uy),
        radii:    Vec2::new(rx, ry),

        fill_color,
        outline_color,
        outline_width,

        corner_radius: 0.,

        kind: KIND_ELLIPSE,
    };
    [
        v(x1, y1, -pad_x, -pad_y), v(x2, y1,  pad_x, -pad_y), v(x2, y2,  pad_x,  pad_y),
        v(x1, y1, -pad_x, -pad_y), v(x2, y2,  pad_x,  pad_y), v(x1, y2, -pad_x,  pad_y),
    ]
}

pub(crate) fn line_vertices(
    a: Vec2,
    b: Vec2,
    color: Color,
    width: f32,
) -> [Vertex; 6] {
    let displacement = b - a;
    let length = displacement.length();
    const EPSILON: f32 = 1e-6;

    if length < EPSILON {
        let radius = width / 2.;
        return ellipse_vertices(
            a.x,
            a.y,
            radius,
            radius,
            color,
            Color::TRANSPARENT,
            0.,
        )
    }

    let dir = displacement.normalize();
    let perp = Vec2::new(-dir.y, dir.x);

    let hw = width / 2.;
    let hl = length / 2.;

    let padding = 1.5;
    let pad_hw = hw + padding;
    let pad_hl = hl + hw + padding;

    let offset_x = dir * pad_hl;
    let offset_y = perp * pad_hw;
    let center = (a + b) / 2.;

    let corners = [
        center - offset_x - offset_y,
        center - offset_x + offset_y,
        center + offset_x + offset_y,
        center + offset_x - offset_y,
    ];

    let uvs = [
        Vec2::new(-pad_hl, -pad_hw),
        Vec2::new(-pad_hl,  pad_hw),
        Vec2::new( pad_hl,  pad_hw),
        Vec2::new( pad_hl, -pad_hw),
    ];

    let v = |i: usize| Vertex {
        position: corners[i],
        uv:       uvs[i],
        radii:    Vec2::new(hl, 0.),

        fill_color: color,
        outline_color: color,
        outline_width: width,

        corner_radius: 0.,

        kind: KIND_LINE,
    };

    [v(0), v(3), v(2), v(0), v(2), v(1)]
}

pub(crate) fn textured_vertices(
    x: f32, y: f32,
    w: f32, h: f32,
    uv_min: Vec2, uv_max: Vec2,
    fill_color: Color,
) -> [Vertex; 6] {
    let xs = [x, x + w];
    let ys = [y, y + h];
    let us = [uv_min.x, uv_max.x];
    let vs = [uv_min.y, uv_max.y];

    let v = |x, y| Vertex {
        position: Vec2::new(xs[x], ys[y]),
        uv:       Vec2::new(us[x], vs[y]),
        radii:    Vec2::ZERO,

        fill_color,
        outline_color: Color::TRANSPARENT,
        outline_width: 0.,

        corner_radius: 0.,

        kind: KIND_TEXTURED,
    };

    [
        v(0, 0), v(1, 0), v(1, 1),
        v(0, 0), v(1, 1), v(0, 1),
    ]
}

pub(crate) fn canvas_vertices(
    x: f32, y: f32,
    w: f32, h: f32,
    uv_min: Vec2, uv_max: Vec2,
    fill_color: Color
) -> [Vertex; 6] {
    let mut vertices = textured_vertices(x, y, w, h, uv_min, uv_max, fill_color);
    for v in &mut vertices {
        v.kind = KIND_CANVAS;
    }
    vertices
}
