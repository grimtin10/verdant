use std::f32::consts::TAU;

use crate::{shapes::{Ellipse, Line, Rect}, vec::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersections<const N: usize> {
    pub points: [Vec2; N],
    len: usize,
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

    pub fn push(&mut self, point: Vec2) -> bool {
        if self.len < N {
            self.points[self.len] = point;
            self.len += 1;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.points = [Vec2::ZERO; N];
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> IntoIterator for Intersections<N> {
    type Item = Vec2;
    type IntoIter = std::array::IntoIter<Vec2, N>;
    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

impl<const N: usize> Extend<Vec2> for Intersections<N> {
    fn extend<T: IntoIterator<Item = Vec2>>(&mut self, iter: T) {
        for pt in iter {
            self.push(pt);
        }
    }
}

pub trait Intersect<T = Self> {
    /// The container type used to return intersection points for this shape pair.
    type Points;

    /// Checks if the target is completely inside this shape.
    fn contains(&self, other: &T) -> bool;

    fn intersection_points(&self, other: &T) -> Option<Self::Points>;

    /// Checks if this shape intersects with the target.
    fn intersects(&self, other: &T) -> bool {
        self.contains(other) || self.intersection_points(other).is_some()
    }
}

impl Rect {
    /// Returns the corners of the rectangle as an array of points.
    pub fn corners(&self) -> [Vec2; 4] {
        [
            self.position,
            Vec2::new(self.position.x + self.size.x, self.position.y),
            Vec2::new(self.position.x + self.size.x, self.position.y + self.size.y),
            Vec2::new(self.position.x, self.position.y + self.size.y),
        ]
    }

    /// Returns the edges of the rectangle as an array of lines.
    pub fn edges(&self) -> [Line; 4] {
        let top_left = self.position;
        let top_right = Vec2::new(self.position.x + self.size.x, self.position.y);
        let bottom_right = Vec2::new(self.position.x + self.size.x, self.position.y + self.size.y);
        let bottom_left = Vec2::new(self.position.x, self.position.y + self.size.y);

        [
            Line::between(top_left.x, top_left.y, top_right.x, top_right.y),
            Line::between(top_right.x, top_right.y, bottom_right.x, bottom_right.y),
            Line::between(bottom_right.x, bottom_right.y, bottom_left.x, bottom_left.y),
            Line::between(bottom_left.x, bottom_left.y, top_left.x, top_left.y),
        ]
    }
}

impl Ellipse {
    /// Transforms a world-space point into unit-circle space.
    pub fn to_unit_space(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            (point.x - self.position.x) / self.size.x,
            (point.y - self.position.y) / self.size.y,
        )
    }

    /// Transforms a unit-circle-space point back into world-space.
    pub fn to_world_space(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x * self.size.x + self.position.x,
            point.y * self.size.y + self.position.y,
        )
    }

    /// Returns whether the ellipse is a circle.
    pub fn is_circle(&self) -> bool {
        (self.size.x - self.size.y).abs() <= f32::EPSILON
    }
}

impl Intersect<Vec2> for Vec2 {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < f32::EPSILON && (self.y - other.y).abs() < f32::EPSILON
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
    fn intersects(&self, other: &Line) -> bool { other.intersects(self) }
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
        let d = self.end - self.start;
        let v = *other - self.start;

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
        self.contains(&other.start) && self.contains(&other.end)
    }

    fn intersects(&self, other: &Line) -> bool {
        let s1 = self.start;
        let e1 = self.end;
        let s2 = other.start;
        let e2 = other.end;

        let orientation = |a: Vec2, b: Vec2, c: Vec2| {
            let val = (b - a).cross(c - a);
            if val > 1e-5 { 1 } else if val < 1e-5 { -1 } else { 0 }
        };

        let on_segment = |p: Vec2, q: Vec2, r: Vec2| {
            q.x >= p.x.min(r.x) && q.x <= p.x.max(r.x) && q.y >= p.y.min(r.y) && q.y <= p.y.max(r.y)
        };

        let o1 = orientation(s1, e1, s2);
        let o2 = orientation(s1, e1, e2);
        let o3 = orientation(s2, e2, s1);
        let o4 = orientation(s2, e2, e1);

        if o1 != o2 && o3 != o4 {
            return true;
        }

        if o1 == 0 && on_segment(s1, e2, e1) { return true; }
        if o2 == 0 && on_segment(s1, e2, e1) { return true; }
        if o3 == 0 && on_segment(s2, e1, e2) { return true; }
        if o4 == 0 && on_segment(s2, e1, e2) { return true; }

        false
    }

    fn intersection_points(&self, other: &Line) -> Option<Self::Points> {
        let p = self.start;
        let r = self.end - self.start;
        let q = other.start;
        let s = other.end - other.start;

        let r_cross_s = r.cross(s);
        let q_p = q - p;

        if r_cross_s.abs() < 1e-6 { return None; }

        let t = q_p.cross(r) / r_cross_s;
        let u = q_p.cross(s) / r_cross_s;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some(Intersections::from([p + t * r]))
        } else {
            None
        }
    }
}

impl Intersect<Rect> for Line {
    type Points = Intersections<8>;

    fn contains(&self, _: &Rect) -> bool { false }
    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> { other.intersection_points(self) }

    fn intersects(&self, other: &Rect) -> bool {
        let min = self.start.min(self.end);
        let max = self.start.max(self.end);

        let pos = other.position;
        let size = other.size;

        if max.x < pos.x || min.x > pos.x + size.x
        || max.y < pos.y || min.y > pos.y + size.y {
            return false;
        }

        if other.contains(&self.start) || other.contains(&self.end) {
            return true;
        }

        other.edges().into_iter().any(|e| e.intersects(self))
    }
}

impl Intersect<Ellipse> for Line {
    type Points = Intersections<2>;

    fn contains(&self, _: &Ellipse) -> bool { false }

    fn intersects(&self, other: &Ellipse) -> bool {
        if other.is_circle() {
            let r = other.size.x;
            let d = self.end - self.start;
            let f = self.start - other.position;

            let a = d.length_squared();
            let b = 2.0 * f.dot(d);
            let c = f.length_squared() - r * r;

            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 { return false }

            let t_min = -b / (2.0 * a);
            return (0.0..=1.0).contains(&t_min) || self.contains(&other.position);
        }

        let start = other.to_unit_space(self.start);
        let end = other.to_unit_space(self.end);
        let d = end - start;
        let f = start;

        let a = d.length_squared();
        let b = 2.0 * f.dot(d);
        let c = f.length_squared() - 1.0;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 { return false }

        let t_min = -b / (2.0 * a);
        (0.0..=1.0).contains(&t_min) || start.length_squared() <= 1.0 || end.length_squared() <= 1.0
    }

    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> {
        let mut list = Intersections::new();

        if other.is_circle() {
           let r = other.size.x;
           let d = self.end - self.start;
           let f = self.start - other.position;

           let a = d.length_squared();
           let b = 2.0 * f.dot(d);
           let c = f.length_squared() - r * r;

           let discriminant = b * b - 4.0 * a * c;
           if discriminant < 0.0 { return None }

           let disc_sqrt = discriminant.sqrt();
           let t1 = (-b - disc_sqrt) / (2.0 * a);
           let t2 = (-b + disc_sqrt) / (2.0 * a);

           if (0.0..=1.0).contains(&t1) {
               list.push(other.to_world_space(self.start + t1 * d));
           }
           if (0.0..=1.0).contains(&t2) {
               list.push(other.to_world_space(self.start + t2 * d));
           }
        } else {
            let start = other.to_unit_space(self.start);
            let end = other.to_unit_space(self.end);

            let d = end - start;
            let f = start;

            let a = d.length_squared();
            let b = 2.0 * f.dot(d);
            let c = f.length_squared() - 1.0;

            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 { return None }

            let disc_sqrt = discriminant.sqrt();
            let t1 = (-b - disc_sqrt) / (2.0 * a);
            let t2 = (-b + disc_sqrt) / (2.0 * a);

            if (0.0..=1.0).contains(&t1) {
                list.push(other.to_world_space(start + t1 * d));
            }
            if (0.0..=1.0).contains(&t2) {
                list.push(other.to_world_space(start + t2 * d));
            }
        }

        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Vec2> for Rect {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        other.x >= self.position.x && other.x <= self.position.x + self.size.x
        && other.y >= self.position.y && other.y <= self.position.y + self.size.y
    }

    fn intersects(&self, other: &Vec2) -> bool {
        self.contains(other)
    }

    fn intersection_points(&self, other: &Vec2) -> Option<Self::Points> {
        if self.contains(other) {
            Some([*other].into())
        } else {
            None
        }
    }
}

impl Intersect<Line> for Rect {
    type Points = Intersections<8>;

    fn contains(&self, other: &Line) -> bool {
        self.contains(&other.start) && self.contains(&other.end)
    }

    fn intersects(&self, other: &Line) -> bool {
        if self.contains(&other.start) || self.contains(&other.end) {
            return true
        }

        self.edges().iter().any(|edge| edge.intersects(other))
    }

    fn intersection_points(&self, other: &Line) -> Option<Self::Points> {
        let mut list = Intersections::new();

        for edge in self.edges() {
            if let Some(points) = edge.intersection_points(other) {
                list.extend(points);
            }
        }

        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Rect> for Rect {
    type Points = Intersections<8>;

    fn contains(&self, other: &Rect) -> bool {
        let p1 = self.position;
        let p2 = other.position;
        let s1 = self.size;
        let s2 = other.size;

        p1.x <= p2.x
        && p1.y <= p2.y
        && p2.x + s2.x <= p1.x + s1.x
        && p2.y + s2.y <= p1.y + s1.y
    }

    fn intersects(&self, other: &Rect) -> bool {
        let p1 = self.position;
        let p2 = other.position;
        let s1 = self.size;
        let s2 = other.size;

        p1.x <= p2.x + s2.x
        && p1.y <= p2.y + s2.y
        && p2.x <= p1.x + s1.x
        && p2.y <= p1.y + s1.y
    }

    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> {
        if !self.intersects(other) { return None }
        let mut list = Intersections::new();
        for edge_a in self.edges() {
            for edge_b in other.edges() {
                if let Some(points) = edge_a.intersection_points(&edge_b) {
                    list.extend(points);
                }
            }
        }
        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Ellipse> for Rect {
    type Points = Intersections<8>;

    fn contains(&self, other: &Ellipse) -> bool {
        let p1 = self.position;
        let p2 = other.position;
        let s1 = self.size;
        let s2 = other.size;

        p2.x - s2.x >= p1.x
        && p2.y - s2.y >= p1.y
        && p1.x + s1.x >= p2.x + s2.x
        && p1.y + s1.y >= p2.y + s2.y
    }

    fn intersects(&self, other: &Ellipse) -> bool {
        if other.is_circle() {
            let r = other.size.x;
            let closest = other.position.clamp(self.position, self.position + self.size);

            let d = (closest - other.position).length_squared();
            return d <= r * r
        }

        if self.contains(&other.position) { return true; }
        self.edges().iter().any(|e| e.intersects(other))
    }

    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> {
        let mut list = Intersections::new();

        for edge in self.edges() {
            if let Some(points) = edge.intersection_points(other) {
                list.extend(points);
            }
        }

        if list.is_empty() { None } else { Some(list) }
    }
}

impl Intersect<Vec2> for Ellipse {
    type Points = Intersections<1>;

    fn contains(&self, other: &Vec2) -> bool {
        if self.is_circle() {
            (*other - self.position).length_squared() <= self.size.x * self.size.x
        } else {
            self.to_unit_space(*other).length_squared() <= 1.0
        }
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

impl Intersect<Line> for Ellipse {
    type Points = Intersections<2>;

    fn contains(&self, other: &Line) -> bool {
        self.contains(&other.start) && self.contains(&other.end)
    }

    fn intersects(&self, other: &Line) -> bool { other.intersects(self) }
    fn intersection_points(&self, other: &Line) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Rect> for Ellipse {
    type Points = Intersections<8>;

    fn contains(&self, other: &Rect) -> bool {
        other.corners().iter().all(|c| self.contains(c))
    }

    fn intersects(&self, other: &Rect) -> bool { other.intersects(self) }
    fn intersection_points(&self, other: &Rect) -> Option<Self::Points> { other.intersection_points(self) }
}

impl Intersect<Ellipse> for Ellipse {
    type Points = Intersections<4>;

    fn contains(&self, other: &Ellipse) -> bool {
        if self.is_circle() && other.is_circle() {
            let r1 = self.size.x;
            let r2 = other.size.x;
            if r1 < r2 {
                return false;
            }

            let d = r1 - r2;
            return (other.position - self.position).length_squared() <= d * d;
        }

        let position = self.to_unit_space(other.position);
        let size = other.size / self.size;
        position.x.abs() + size.x <= 1.0 && position.y.abs() + size.y <= 1.0
    }

    fn intersects(&self, other: &Ellipse) -> bool {
        if self.is_circle() && other.is_circle() {
            let r_sum = self.size.x + other.size.x;
            return (other.position - self.position).length_squared() <= r_sum * r_sum;
        }

        let position = self.to_unit_space(other.position);
        let size = other.size / self.size;
        let max_r = 1.0 + size.x.max(size.y);
        position.length_squared() <= max_r * max_r
    }

    fn intersection_points(&self, other: &Ellipse) -> Option<Self::Points> {
        if !self.intersects(other) { return None }

        if self.is_circle() && other.is_circle() {
            let r1 = self.size.x;
            let r2 = other.size.x;
            let d_vec = other.position - self.position;
            let d2 = d_vec.length_squared();
            let d = d2.sqrt();

            if d > r1 + r2 || d < (r1 - r2).abs() || d == 0.0 {
                return None;
            }

            let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
            let h = (r1 * r1 - a * a).max(0.0).sqrt();

            let p2 = self.position + d_vec * (a / d);
            let offset = Vec2::new(h * (d_vec.y / d), -h * (d_vec.x / d));

            let mut list = Intersections::new();

            list.push(p2 + offset);
            if h > 1e-5 {
                list.push(p2 - offset);
            }

            return Some(list)
        }

        let mut list = Intersections::new();
        let samples = 16;
        let mut prev_point = other.position + Vec2::new(other.size.x, 0.0);
        let mut prev_inside = self.contains(&prev_point);

        for i in 1..=samples {
            let angle = (i as f32 / samples as f32) * TAU;
            let curr_point = other.position + Vec2::new(other.size.x * angle.cos(), other.size.y * angle.sin());
            let curr_inside = self.contains(&curr_point);

            if prev_inside != curr_inside {
                let line = Line::between(prev_point.x, prev_point.y, curr_point.x, curr_point.y);
                if let Some(points) = line.intersection_points(self) {
                    list.extend(points);
                }
            }
            prev_point = curr_point;
            prev_inside = curr_inside;
        }

        if list.is_empty() { None } else { Some(list) }
    }
}
