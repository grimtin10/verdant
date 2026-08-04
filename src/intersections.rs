// this was much more of a rabbithole than i was expecting
// also some might say "but wait i thought verdant was meant to be a rendering library! not a game engine!"
// i'd say this doesn't constitute a game engine, and that verdant is just a very weird rendering library
// i wanted intersection tests, so i added intersection tests!
// "are you going to add physics?" no, but you could build it using these

use std::{f32::consts::FRAC_1_SQRT_2, f64::consts::TAU, iter::Take, ops::Index};

use crate::{shapes::{Ellipse, Line, Rect}, transform::Transform2d, vec::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersections<const N: usize> {
    pub points: [Vec2; N],
    len: usize,
}

impl<const N: usize> Index<usize> for Intersections<N> {
    type Output = Vec2;
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("Index {} out of bounds for Intersections of len {}", index, self.len);
        }
        &self.points[index]
    }
}

impl<const N: usize> Default for Intersections<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<[Vec2; N]> for Intersections<N> {
    fn from(points: [Vec2; N]) -> Self {
        Self {
            points,
            len: N,
        }
    }
}

impl<const N: usize> From<Intersections<N>> for [Vec2; N] {
    fn from(intersections: Intersections<N>) -> Self {
        intersections.points
    }
}

impl<const N: usize> Intersections<N> {
    pub const fn new() -> Self {
        Self {
            points: [Vec2::ZERO; N],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, point: Vec2) -> bool {
        if self.len < N {
            self.points[self.len] = point;
            self.len += 1;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)] pub fn len(&self) -> usize { self.len }
    #[inline(always)] pub fn is_empty(&self) -> bool { self.len == 0 }
}

impl<const N: usize> IntoIterator for Intersections<N> {
    type Item = Vec2;
    type IntoIter = Take<std::array::IntoIter<Vec2, N>>;
    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter().take(self.len)
    }
}

impl<const N: usize> Extend<Vec2> for Intersections<N> {
    fn extend<T: IntoIterator<Item = Vec2>>(&mut self, iter: T) {
        for point in iter {
            self.push(point);
        }
    }
}

pub trait Intersect<T> {
    /// The container type used to return intersection points for this shape pair.
    type Points;

    /// Checks if the target is completely inside this shape.
    fn contains(&self, other: &T) -> bool;

    /// Returns all intersection points between this shape and the target.
    fn intersection_points(&self, other: &T) -> Option<Self::Points>;

    /// Checks if this shape intersects with the target.
    fn intersects(&self, other: &T) -> bool {
        self.contains(other) || self.intersection_points(other).is_some()
    }
}

impl Line {
    /// Returns the start point of the line in world-space, transformed by `self.transform`.
    pub fn world_start(&self) -> Vec2 {
        self.transform.transform_point(self.start)
    }

    /// Returns the end point of the line in world-space, transformed by `self.transform`.
    pub fn world_end(&self) -> Vec2 {
        self.transform.transform_point(self.end)
    }

    /// Returns the line as a [`Line`] struct in world-space.
    /// Start and end points are transformed by `self.transform`, and the new transform is set to [`Transform2d::identity()`].
    pub fn to_world(&self) -> Self {
        Self {
            start: self.world_start(),
            end: self.world_end(),
            style: self.style,
            transform: Transform2d::identity(),
        }
    }
}

impl Rect {
    /// Returns the corners of the rectangle in local-space (axis-aligned).
    #[inline(always)]
    pub fn local_corners(&self) -> [Vec2; 4] {
        [
            self.position,
            Vec2::new(self.position.x + self.size.x, self.position.y),
            Vec2::new(self.position.x + self.size.x, self.position.y + self.size.y),
            Vec2::new(self.position.x, self.position.y + self.size.y),
        ]
    }

    /// Returns the corners of the rectangle in world-space, transformed by `self.transform`.
    #[inline(always)]
    pub fn corners(&self) -> [Vec2; 4] {
        self.local_corners().map(|p| self.transform.transform_point(p))
    }

    /// Returns the edges of the rectangle as an array of lines.
    /// Points are in world-space, transformed by `self.transform`.
    #[inline(always)]
    pub fn edges(&self) -> [Line; 4] {
        let [tl, tr, br, bl] = self.corners();
        [
            Line::between(tl.x, tl.y, tr.x, tr.y),
            Line::between(tr.x, tr.y, br.x, br.y),
            Line::between(br.x, br.y, bl.x, bl.y),
            Line::between(bl.x, bl.y, tl.x, tl.y),
        ]
    }

    /// Returns the bounding circle of this [`Rect`].
    pub fn bounding_circle(&self) -> (Vec2, f32) {
        let center = self.transform.transform_point(self.position + self.size * 0.5);
        let [m11, m21, m12, m22, _, _] = self.transform.matrix;
        let sx = (m11 * m11 + m21 * m21).sqrt();
        let sy = (m12 * m12 + m22 * m22).sqrt();
        let half_w = self.size.x * 0.5;
        let half_h = self.size.y * 0.5;
        let radius = (half_w * half_w + half_h * half_h).sqrt() * sx.max(sy);
        (center, radius)
    }
}

impl Ellipse {
    /// Transforms a local-space point into unit-circle-space.
    pub fn local_to_unit(&self, point: impl Into<Vec2>) -> Vec2 {
        let point = point.into();
        Vec2::new(
            (point.x - self.position.x) / self.size.x,
            (point.y - self.position.y) / self.size.y,
        )
    }

    /// Transforms a unit-circle-space point back into local-space.
    pub fn unit_to_local(&self, point: impl Into<Vec2>) -> Vec2 {
        let point = point.into();
        Vec2::new(
            point.x * self.size.x + self.position.x,
            point.y * self.size.y + self.position.y,
        )
    }

    /// Transforms a world-space point into unit-circle-space.
    pub fn world_to_unit(&self, point: impl Into<Vec2>) -> Option<Vec2> {
        let inv = self.transform.inverse()?;
        Some(self.local_to_unit(inv.transform_point(point)))
    }

    /// Transforms a unit-circle-space point back into world-space.
    pub fn unit_to_world(&self, point: impl Into<Vec2>) -> Vec2 {
        self.transform.transform_point(self.unit_to_local(point))
    }

    fn circle_params(&self) -> (Vec2, f32) {
        let center = self.transform.transform_point(self.position);
        let [m11, m21, _, _, _, _] = self.transform.matrix;
        let sx = (m11 * m11 + m21 * m21).sqrt();
        (center, self.size.x * sx)
    }

    /// Checks if this [`Ellipse`] is a perfect circle.
    /// Takes into account transformations.
    pub fn is_circle(&self) -> bool {
        let [m11, m21, m12, m22, _, _] = self.transform.matrix;
        let sx = m11 * m11 + m21 * m21;
        let sy = m12 * m12 + m22 * m22;

        let rx = self.size.x * self.size.x * sx;
        let ry = self.size.y * self.size.y * sy;

        (rx - ry).abs() < 1e-5
    }

    /// Returns the center and radius of the bounding circle of this [`Ellipse`].
    pub fn bounding_circle(&self) -> (Vec2, f32) {
        let center = self.transform.transform_point(self.position);
        let [m11, m21, m12, m22, _, _] = self.transform.matrix;
        let sx = (m11 * m11 + m21 * m21).sqrt();
        let sy = (m12 * m12 + m22 * m22).sqrt();
        (center, self.size.x.max(self.size.y) * sx.max(sy))
    }

    /// Checks if this [`Ellipse`] fully contains another with 100% mathematical exactness.
    /// Slower than `contains()`, but lossless. You can just use `contains()` for most cases.
    pub fn contains_exact(&self, other: &Ellipse) -> bool {
        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if r1 < r2 { return false; }
        if (c1 - c2).length_squared() > (r1 - r2).powi(2) { return false; }

        if self.intersection_points(other).is_some() { return false; }

        let b_boundary_pt = other.unit_to_world(Vec2::new(1.0, 0.0));
        self.contains(&b_boundary_pt)
    }

    /// Checks if this [`Ellipse`] and another intersect with 100% mathematical exactness.
    /// Slower than `intersects()`, but lossless. You can just use `intersects()` for most cases.
    pub fn intersects_exact(&self, other: &Ellipse) -> bool {
        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if (c1 - c2).length_squared() > (r1 + r2) * (r1 + r2) { return false; }

        if self.is_circle() && other.is_circle() {
            let (_, r1) = self.circle_params();
            let (_, r2) = other.circle_params();
            let d = (c1 - c2).length();
            return d <= r1 + r2 && d >= (r1 - r2).abs();
        }

        if self.intersection_points(other).is_some() { return true; }

        let a_center = self.transform.transform_point(self.position);
        if other.contains(&a_center) { return true; }

        let b_center = other.transform.transform_point(other.position);
        if self.contains(&b_center) { return true; }

        false
    }

    fn get_b_to_a_unit_transform(&self, other: &Ellipse) -> Option<Transform2d> {
        let inv_a = self.transform.inverse()?;

        let to_world_b = Transform2d::scaling(other.size.x, other.size.y)
            .translate(other.position.x, other.position.y)
            .then(other.transform);

        let to_unit_a = Transform2d::translation(-self.position.x, -self.position.y)
            .then(Transform2d::scaling(1.0 / self.size.x, 1.0 / self.size.y));

        Some(to_world_b.then(inv_a).then(to_unit_a))
    }
}

impl Intersect<Vec2> for Vec2 {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < 1e-5 && (self.y - other.y).abs() < 1e-5
    }

    fn intersection_points(&self, other: &Vec2) -> Option<Self::Points> {
        if self.contains(other) {
            Some([*other].into())
        } else {
            None
        }
    }
}

impl Intersect<Line> for Vec2 {
    type Points = Intersections<1>;

    fn contains(&self, _: &Line) -> bool { false }
    fn intersects(&self, other: &Line) -> bool { other.contains(self) }
    fn intersection_points(&self, other: &Line) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Rect> for Vec2 {
    type Points = Intersections<1>;

    fn contains(&self, _: &Rect) -> bool { false }
    fn intersects(&self, other: &Rect) -> bool { other.contains(self) }
    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Ellipse> for Vec2 {
    type Points = Intersections<1>;

    fn contains(&self, _: &Ellipse) -> bool { false }
    fn intersects(&self, other: &Ellipse) -> bool { other.contains(self) }
    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Vec2> for Line {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        let w = self.to_world();
        let d = w.end - w.start;
        let v = *other - w.start;

        if v.cross(d).abs() > 1e-5 { return false; }
        let dot = v.dot(d);
        dot >= 0.0 && dot <= d.length_squared()
    }

    fn intersection_points(&self, other: &Vec2) -> Option<Self::Points> {
        if self.contains(other) {
            Some([*other].into())
        } else {
            None
        }
    }
}

impl Intersect<Line> for Line {
    type Points = Intersections<1>;

    fn contains(&self, other: &Line) -> bool {
        let w = other.to_world();
        self.contains(&w.start) && self.contains(&w.end)
    }

    fn intersects(&self, other: &Line) -> bool {
        let s1 = self.world_start();
        let e1 = self.world_end();
        let s2 = other.world_start();
        let e2 = other.world_end();

        let orientation = |a: Vec2, b: Vec2, c: Vec2| -> i32 {
            let val = (b - a).cross(c - a);
            if val > 1e-5 { 1 } else if val < -1e-5 { -1 } else { 0 }
        };

        let on_segment = |p: Vec2, q: Vec2, r: Vec2| -> bool {
            q.x >= p.x.min(r.x) && q.x <= p.x.max(r.x) &&
            q.y >= p.y.min(r.y) && q.y <= p.y.max(r.y)
        };

        let o1 = orientation(s1, e1, s2);
        let o2 = orientation(s1, e1, e2);
        let o3 = orientation(s2, e2, s1);
        let o4 = orientation(s2, e2, e1);

        if o1 != o2 && o3 != o4 { return true; }
        if o1 == 0 && on_segment(s1, s2, e1) { return true; }
        if o2 == 0 && on_segment(s1, e2, e1) { return true; }
        if o3 == 0 && on_segment(s2, s1, e2) { return true; }
        if o4 == 0 && on_segment(s2, e1, e2) { return true; }
        false
    }

    fn intersection_points(&self, other: &Line) -> Option<Self::Points> {
        let p = self.world_start();
        let r = self.world_end() - p;
        let q = other.world_start();
        let s = other.world_end() - q;

        let r_cross_s = r.cross(s);
        let q_minus_p = q - p;

        if r_cross_s.abs() < 1e-5 {
            return None;
        }

        let t = q_minus_p.cross(s) / r_cross_s;
        let u = q_minus_p.cross(r) / r_cross_s;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some([p + r * t].into())
        } else {
            None
        }
    }
}

impl Intersect<Rect> for Line {
    type Points = Intersections<2>;

    fn contains(&self, _: &Rect) -> bool { false }
    fn intersects(&self, other: &Rect) -> bool { other.intersects(self) }
    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Ellipse> for Line {
    type Points = Intersections<2>;

    fn contains(&self, _: &Ellipse) -> bool { false }
    fn intersects(&self, other: &Ellipse) -> bool { other.intersects(self) }
    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Vec2> for Rect {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };
        let local = inv.transform_point(*other);
        local.x >= self.position.x && local.x <= self.position.x + self.size.x &&
        local.y >= self.position.y && local.y <= self.position.y + self.size.y
    }

    fn intersects(&self, other: &Vec2) -> bool { self.contains(other) }

    fn intersection_points(&self, other: &Vec2) -> Option<Self::Points> {
        if self.contains(other) {
            Some([*other].into())
        } else {
            None
        }
   }
}

fn liang_barsky(rect: &Rect, start: Vec2, end: Vec2) -> Option<(f32, f32)> {
    let d = end - start;

    let min = rect.position;
    let max = rect.position + rect.size;

    let mut t0 = 0f32;
    let mut t1 = 1f32;

    let p = [-d.x, d.x, -d.y, d.y];
    let q = [start.x - min.x, max.x - start.x, start.y - min.y, max.y - start.y];

    for i in 0..4 {
        if p[i].abs() < 1e-5 {
            if q[i] < 0.0 { return None }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > t1 { return None; }
                t0 = t0.max(t);
            } else {
                if t < t0 { return None; }
                t1 = t1.min(t);
            }
        }
    }

    Some((t0, t1))
}

impl Intersect<Line> for Rect {
    type Points = Intersections<2>;

    fn contains(&self, other: &Line) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };
        let w = other.to_world();

        let s = inv.transform_point(w.start);
        let e = inv.transform_point(w.end);

        s.x >= self.position.x && s.x <= self.position.x + self.size.x &&
        s.y >= self.position.y && s.y <= self.position.y + self.size.y &&
        e.x >= self.position.x && e.x <= self.position.x + self.size.x &&
        e.y >= self.position.y && e.y <= self.position.y + self.size.y
    }

    fn intersects(&self, other: &Line) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };
        let w = other.to_world();

        let start = inv.transform_point(w.start);
        let end = inv.transform_point(w.end);

        let Some((t0, t1)) = liang_barsky(self, start, end) else { return false };
        t0 <= t1
    }

    fn intersection_points(&self, other: &Line) -> Option<Self::Points> {
        let inv = self.transform.inverse()?;
        let w = other.to_world();

        let start = inv.transform_point(w.start);
        let end = inv.transform_point(w.end);

        let (t0, t1) = liang_barsky(self, start, end)?;

        if t0 <= t1 {
            let mut list = Intersections::new();
            if t0 > 0.0 && t0 <= 1.0 {
                list.push(self.transform.transform_point(start + (end - start) * t0));
            }
            if (0.0..=1.0).contains(&t1) {
                list.push(self.transform.transform_point(start + (end - start) * t1));
            }

            if list.is_empty() { None } else { Some(list) }
        } else {
            None
        }
    }
}

impl Intersect<Rect> for Rect {
    type Points = Intersections<8>;

    fn contains(&self, other: &Rect) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };

        other.corners().into_iter().all(|c| {
            let p = inv.transform_point(c);
            p.x >= self.position.x && p.x <= self.position.x + self.size.x &&
            p.y >= self.position.y && p.y <= self.position.y + self.size.y
        })
    }

    fn intersects(&self, other: &Rect) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };

        let [p0, p1, p2, p3] = other.corners().map(|c| inv.transform_point(c));
        let min = p0.min(p1).min(p2).min(p3);
        let max = p0.max(p1).max(p2).max(p3);

        if max.x < self.position.x || min.x > self.position.x + self.size.x
        || max.y < self.position.y || min.y > self.position.y + self.size.y {
            return false;
        }

        let Some(inv) = other.transform.inverse() else { return false };

        let [p0, p1, p2, p3] = self.corners().map(|c| inv.transform_point(c));
        let min = p0.min(p1).min(p2).min(p3);
        let max = p0.max(p1).max(p2).max(p3);

        if max.x < other.position.x || min.x > other.position.x + other.size.x
        || max.y < other.position.y || min.y > other.position.y + other.size.y {
            return false;
        }

        true
    }

    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> {
        if !self.intersects(other) { return None; }

        let mut list = Intersections::new();
        let edges_a = self.edges();
        let edges_b = other.edges();

        for a in edges_a {
            for b in edges_b {
                if let Some(points) = a.intersection_points(&b) { list.extend(points); }
            }
        }
        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Ellipse> for Rect {
    type Points = Intersections<8>;

    fn contains(&self, other: &Ellipse) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };

        let other_center = inv.transform_point(other.transform.transform_point(other.position));

        let vx = inv.transform_vector(other.transform.transform_vector(Vec2::new(other.size.x, 0.0)));
        let vy = inv.transform_vector(other.transform.transform_vector(Vec2::new(0.0, other.size.y)));

        let rx = (vx.x * vx.x + vy.x * vy.x).sqrt();
        let ry = (vx.y * vx.y + vy.y * vy.y).sqrt();

        other_center.x - rx >= self.position.x &&
        other_center.x + rx <= self.position.x + self.size.x &&
        other_center.y - ry >= self.position.y &&
        other_center.y + ry <= self.position.y + self.size.y
    }

    fn intersects(&self, other: &Ellipse) -> bool {
        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if (c1 - c2).length_squared() > (r1 + r2) * (r1 + r2) { return false }

        let other_center = other.transform.transform_point(other.position);
        if self.contains(&other_center) { return true }

        let Some(other_inv) = other.transform.inverse() else { return false };
        let unit_corners = self.corners().map(|c| {
            other.local_to_unit(other_inv.transform_point(c))
        });

        let dist_sq_to_origin = |a: Vec2, b: Vec2| -> f32 {
            let ab = b - a;
            let len_sq = ab.dot(ab);
            if len_sq < 1e-10 { return a.length_squared(); }
            let t = (-a.dot(ab) / len_sq).clamp(0.0, 1.0);
            (a + ab * t).length_squared()
        };

        for i in 0..4 {
            if dist_sq_to_origin(unit_corners[i], unit_corners[(i + 1) % 4]) <= 1.0 {
                return true;
            }
        }

        false
    }

    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> {
        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if (c1 - c2).length_squared() > (r1 + r2) * (r1 + r2) { return None }

        let other_inv = other.transform.inverse()?;
        let unit_corners = self.corners().map(|c| {
            other.local_to_unit(other_inv.transform_point(c))
        });

        let mut list = Intersections::new();

        for i in 0..4 {
            let c1 = unit_corners[i];
            let c2 = unit_corners[(i + 1) % 4];
            let d = c2 - c1;

            let a = d.dot(d);
            if a < 1e-10 { continue }

            let b = 2.0 * c1.dot(d);
            let c = c1.dot(c1) - 1.0;
            let discriminant = b * b - 4.0 * a * c;

            if discriminant < 0.0 { continue }

            let sqrt_disc = discriminant.sqrt();
            let t1 = (-b - sqrt_disc) / (2.0 * a);
            let t2 = (-b + sqrt_disc) / (2.0 * a);

            if (0.0..=1.0).contains(&t1) {
                let point = c1 + d * t1;
                list.push(other.unit_to_world(point));
            }
            if (0.0..=1.0).contains(&t2) {
                let point = c1 + d * t2;
                list.push(other.unit_to_world(point));
            }
        }

        if !list.is_empty() { Some(list) } else { None }
    }
}

impl Intersect<Vec2> for Ellipse {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool { self.world_to_unit(*other).is_some_and(|p| p.length_squared() <= 1.0) }
    fn intersects(&self, other: &Vec2) -> bool { self.contains(other) }
    fn intersection_points(&self, other: &Vec2) -> Option<Self::Points> {
        if self.contains(other) {
            Some([*other].into())
        } else {
            None
        }
    }
}

impl Intersect<Line> for Ellipse {
    type Points = Intersections<2>;

    fn contains(&self, other: &Line) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };
        let w = other.to_world();

        let start = self.local_to_unit(inv.transform_point(w.start));
        let end = self.local_to_unit(inv.transform_point(w.end));

        start.length_squared() <= 1.0 && end.length_squared() <= 1.0
    }

    fn intersects(&self, other: &Line) -> bool {
        let Some(inv) = self.transform.inverse() else { return false };
        let w = other.to_world();
        let u1 = self.local_to_unit(inv.transform_point(w.start));
        let u2 = self.local_to_unit(inv.transform_point(w.end));

        let l1 = u1.length_squared();
        if l1 <= 1.0 { return true }
        let l2 = u2.length_squared();
        if l2 <= 1.0 { return true }

        let d = u2 - u1;
        let v = d.length_squared();
        let u = -u1.dot(d);
        if u >= 0.0 && u <= v {
            v * (l1 - 1.0) <= u * u
        } else {
            false
        }
    }

    fn intersection_points(&self, other: &Line) -> Option<Self::Points> {
        let inv = self.transform.inverse()?;
        let w = other.to_world();
        let u1 = self.local_to_unit(inv.transform_point(w.start));
        let u2 = self.local_to_unit(inv.transform_point(w.end));

        let d = u2 - u1;
        let a = d.length_squared();
        if a < 1e-10 { return None }

        let b = 2.0 * u1.dot(d);
        let c = u1.length_squared() - 1.0;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 { return None }

        let sqrt_disc = discriminant.sqrt();
        let inv_2a = 1.0 / (2.0 * a);

        let mut list = Intersections::new();
        let t1 = (-b - sqrt_disc) * inv_2a;
        let t2 = (-b + sqrt_disc) * inv_2a;

        if (0.0..=1.0).contains(&t1) { list.push(self.unit_to_world(u1 + d * t1)); }
        if (0.0..=1.0).contains(&t2) { list.push(self.unit_to_world(u1 + d * t2)); }

        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Rect> for Ellipse {
    type Points = Intersections<8>;

    fn contains(&self, other: &Rect) -> bool { other.corners().iter().all(|c| self.contains(c)) }
    fn intersects(&self, other: &Rect) -> bool { other.intersects(self) }
    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> { other.intersection_points(self) }
}

const UNIT_CIRCLE_16: [(f32, f32); 16] = [
    (1.0, 0.0), (0.9238795, 0.3826834), (FRAC_1_SQRT_2, FRAC_1_SQRT_2), (0.3826834, 0.9238795),
    (0.0, 1.0), (-0.3826834, 0.9238795), (-FRAC_1_SQRT_2, FRAC_1_SQRT_2), (-0.9238795, 0.3826834),
    (-1.0, 0.0), (-0.9238795, -0.3826834), (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (-0.3826834, -0.9238795),
    (0.0, -1.0), (0.3826834, -0.9238795), (FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (0.9238795, -0.3826834)
];

const UNIT_CIRCLE_32: [(f32, f32); 32] = [
    (1.0, 0.0), (0.9807853, 0.1950903), (0.9238795, 0.3826834), (0.8314696, 0.5555702),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2), (0.5555702, 0.8314696), (0.3826834, 0.9238795), (0.1950903, 0.9807853),
    (0.0, 1.0), (-0.1950903, 0.9807853), (-0.3826834, 0.9238795), (-0.5555702, 0.8314696),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2), (-0.8314696, 0.5555702), (-0.9238795, 0.3826834), (-0.9807853, 0.1950903),
    (-1.0, 0.0), (-0.9807853, -0.1950903), (-0.9238795, -0.3826834), (-0.8314696, -0.5555702),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (-0.5555702, -0.8314696), (-0.3826834, -0.9238795), (-0.1950903, -0.9807853),
    (0.0, -1.0), (0.1950903, -0.9807853), (0.3826834, -0.9238795), (0.5555702, -0.8314696),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (0.8314696, -0.5555702), (0.9238795, -0.3826834), (0.9807853, -0.1950903),
];

impl Intersect<Ellipse> for Ellipse {
    type Points = Intersections<4>;

    fn contains(&self, other: &Ellipse) -> bool {
        if self.is_circle() && other.is_circle() {
            let (c1, r1) = self.circle_params();
            let (c2, r2) = other.circle_params();

            if r1 < r2 { return false }
            return (c2 - c1).length_squared() <= (r1 - r2) * (r1 - r2)
        }

        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if r1 < r2 { return false }
        if (c1 - c2).length_squared() > (r1 + r2) * (r1 + r2) { return false }

        let Some(b_to_a) = self.get_b_to_a_unit_transform(other) else { return false };

        for v in UNIT_CIRCLE_32 {
            let u = b_to_a.transform_point(v);
            if u.length_squared() > 1.0 { return false }
        }
        true
    }

    fn intersects(&self, other: &Ellipse) -> bool {
        let (c1, r1) = self.bounding_circle();
        let (c2, r2) = other.bounding_circle();
        if (c1 - c2).length_squared() > (r1 + r2) * (r1 + r2) { return false }

        if self.is_circle() && other.is_circle() {
            let (_, r1) = self.circle_params();
            let (_, r2) = other.circle_params();
            let d = (c1 - c2).length();
            return d <= r1 + r2 && d >= (r1 - r2).abs();
        }

        let a_center = self.transform.transform_point(self.position);
        if other.contains(&a_center) { return true }
        let b_center = other.transform.transform_point(other.position);
        if self.contains(&b_center) { return true }

        let Some(b_to_a) = self.get_b_to_a_unit_transform(other) else { return false };

        let mut prev_u = b_to_a.transform_point(UNIT_CIRCLE_16[0]);
        for i in 1..=16 {
            let v = UNIT_CIRCLE_16[i % 16];
            let u = b_to_a.transform_point(v);

            let d = u - prev_u;
            let len_sq = d.length_squared();
            if len_sq > 1e-10 {
                let t = (-prev_u.dot(d) / len_sq).clamp(0.0, 1.0);
                let closest = prev_u + d * t;
                if closest.length_squared() <= 1.0 { return true }
            }
            prev_u = u;
        }
        false
    }

    // this is a big one
    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> {
        let (bc1, br1) = self.bounding_circle();
        let (bc2, br2) = other.bounding_circle();
        if (bc1 - bc2).length_squared() > (br1 + br2) * (br1 + br2) { return None }

        if self.is_circle() && other.is_circle() {
            let (c1, r1) = self.circle_params();
            let (c2, r2) = other.circle_params();

            let d_vec = c2 - c1;
            let d2 = d_vec.length_squared();
            let d = d2.sqrt();
            if d > r1 + r2 || d < (r1 - r2).abs() || d < 1e-5 { return None }

            let a = (r1 * r1 - r2 * r2 + d2) / (2.0 * d);
            let h = (r1 * r1 - a * a).max(0.0).sqrt();
            let p = c1 + d_vec * (a / d);
            let offset = Vec2::new(h * (d_vec.y / d), -h * (d_vec.x / d));

            let mut list = Intersections::new();
            list.push(p + offset);
            if h > 1e-5 { list.push(p - offset); }
            return Some(list);
        }

        let b_to_a = self.get_b_to_a_unit_transform(other)?;

        let [m11, m21, m12, m22, cx, cy] = b_to_a.matrix;

        let coef_a = m11 * m11 + m21 * m21;
        let coef_b = m12 * m12 + m22 * m22;
        let coef_c = 2.0 * (m11 * m12 + m21 * m22);
        let coef_d = 2.0 * (m11 * cx + m21 * cy);
        let coef_e = 2.0 * (m12 * cx + m22 * cy);
        let coef_f = cx * cx + cy * cy - 1.0;

        let p4 = coef_a - coef_d + coef_f;
        let p3 = 2.0 * (coef_e - coef_c);
        let p2 = 2.0 * (-coef_a + 2.0 * coef_b + coef_f);
        let p1 = 2.0 * (coef_c + coef_e);
        let p0 = coef_a + coef_d + coef_f;

        let p4_is_zero = p4.abs() < 1e-6;
        let p4_eff = if p4_is_zero { 0.0 } else { p4 as f64 };

        let roots = solve_quartic(
            p4_eff, p3 as f64, p2 as f64, p1 as f64, p0 as f64,
        );

        let mut list: Self::Points = Intersections::new();

        for root in roots.iter().flatten() {
            let w = *root as f32;
            let denom = 1.0 + w * w;
            if denom < 1e-10 { continue; }
            let cos_t = (1.0 - w * w) / denom;
            let sin_t = 2.0 * w / denom;
            list.push(other.unit_to_world(Vec2::new(cos_t, sin_t)));
        }

        if p4_is_zero {
            let v = Vec2::new(-1.0, 0.0);
            let u = b_to_a.transform_point(v);
            if (u.length_squared() - 1.0).abs() < 1e-4 {
                list.push(other.unit_to_world(v));
            }
        }

        if list.is_empty() { None } else { Some(list) }
    }
}

// solves a4*x^4 + a3*x^3 + a2*x^2 + a1*x + a0
fn solve_quartic(a4: f64, a3: f64, a2: f64, a1: f64, a0: f64) -> [Option<f64>; 4] {
    if a4.abs() < 1e-12 {
        if a3.abs() < 1e-12 {
            if a2.abs() < 1e-12 { return [None; 4] }
            let disc = a1 * a1 - 4.0 * a2 * a0;
            if disc < 0.0 { return [None; 4] }
            let sd = disc.sqrt();
            let inv_2a2 = 1.0 / (2.0 * a2);
            return [
                Some((-a1 + sd) * inv_2a2),
                Some((-a1 - sd) * inv_2a2),
                None,
                None,
            ];
        }

        let b = a2 / a3;
        let c = a1 / a3;
        let d = a0 / a3;
        let cubic = solve_cubic(b, c, d);
        return [cubic[0], cubic[1], cubic[2], None];
    }

    let b = a3 / a4;
    let c = a2 / a4;
    let d = a1 / a4;
    let e = a0 / a4;

    let p = c - 3.0 * b * b / 8.0;
    let q = d - b * c / 2.0 + b * b * b / 8.0;
    let r = e - b * d / 4.0 + b * b * c / 16.0 - 3.0 * b * b * b * b / 256.0;
    let shift = -b / 4.0;

    if q.abs() < 1e-12 {
        let mut roots = [None; 4];
        let mut idx = 0;
        let disc = p * p - 4.0 * r;
        if disc >= 0.0 {
            let sd = disc.sqrt();
            for y in [(-p + sd) / 2.0, (-p - sd) / 2.0] {
                if y > 1e-12 {
                    let sy = y.sqrt();
                    roots[idx] = Some(sy + shift); idx += 1;
                    roots[idx] = Some(-sy + shift); idx += 1;
                } else if y.abs() < 1e-12 {
                    roots[idx] = Some(shift); idx += 1;
                }
            }
        }
        return roots;
    }

    let cubic_roots = solve_cubic(-p / 2.0, -r, (4.0 * p * r - q * q) / 8.0);

    let Some(y) = cubic_roots.iter().flatten().find(|&&y| 2.0 * y - p > 1e-12) else { return [None; 4] };

    let sqrt_2y_p = (2.0 * y - p).sqrt();
    let inv_sqrt = 1.0 / (2.0 * sqrt_2y_p);

    let mut roots = [None; 4];
    let mut idx = 0;

    let disc1 = (2.0 * y - p) - 4.0 * (y + q * inv_sqrt);
    if disc1 >= 0.0 {
        let sd = disc1.sqrt();
        roots[idx] = Some((sqrt_2y_p + sd) / 2.0 + shift); idx += 1;
        if sd > 1e-10 { roots[idx] = Some((sqrt_2y_p - sd) / 2.0 + shift); idx += 1; }
    }

    let disc2 = (2.0 * y - p) - 4.0 * (y - q * inv_sqrt);
    if disc2 >= 0.0 {
        let sd = disc2.sqrt();
        roots[idx] = Some((-sqrt_2y_p + sd) / 2.0 + shift); idx += 1;
        if sd > 1e-10 { roots[idx] = Some((-sqrt_2y_p - sd) / 2.0 + shift); }
    }

    roots
}

// solves x^3 + bx^2 + cx + d
fn solve_cubic(b: f64, c: f64, d: f64) -> [Option<f64>; 3] {
    let p = c - b * b / 3.0;
    let q = 2.0 * b * b * b / 27.0 - b * c / 3.0 + d;
    let shift = -b / 3.0;

    if p.abs() < 1e-12 && q.abs() < 1e-12 {
        return [Some(shift), None, None];
    }
    if p.abs() < 1e-12 {
        return [Some((-q).cbrt() + shift), None, None]
    }

    let disc = q * q / 4.0 + p * p * p / 27.0;

    if disc > 1e-12 {
        let sd = disc.sqrt();
        let u = (-q / 2.0 + sd).cbrt();
        let v = (-q / 2.0 - sd).cbrt();
        [Some(u + v + shift), None, None]
    } else if disc < -1e-12 {
        let m = 2.0 * (-p / 3.0).sqrt();
        let arg = (3.0 * q / (p * m)).clamp(-1.0, 1.0);
        let theta = arg.acos() / 3.0;
        [
            Some(m * (theta).cos() + shift),
            Some(m * (theta - 2.0 * TAU / 3.0).cos() + shift),
            Some(m * (theta + 2.0 * TAU / 3.0).cos() + shift),
        ]
    } else {
        if q.abs() < 1e-12 {
            [Some(shift), None, None]
        } else {
            let t1 = 3.0 * q / p;
            let t2 = -1.5 * q / p;
            [Some(t1 + shift), Some(t2 + shift), None]
        }
    }
}
