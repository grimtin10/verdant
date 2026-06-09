use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
/// A struct representing a 2-dimensional vector, with `x` and `y` components.
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// A [`Vec2`] with all values set to 0.
    pub const ZERO: Vec2 = Vec2::new(0., 0.);
    /// A [`Vec2`] with all values set to 1.
    pub const ONE:  Vec2 = Vec2::new(1., 1.);

    /// Create a new [`Vec2`] with `x` and `y`.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Get the length of this [`Vec2`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Get a normalized copy of this [`Vec2`].
    pub fn normalize(&self) -> Vec2 {
        *self / self.length()
    }

    /// Get the angle of this [`Vec2`] in radians.
    pub fn angle_rad(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Get the angle of this [`Vec2`] in degrees.
    pub fn angle_deg(&self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }

    /// Get the distance between this [`Vec2`] and `other`.
    pub fn dist(&self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(v: Vec2) -> Self {
        (v.x, v.y)
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(v: (f32, f32)) -> Self {
        Self::new(v.0, v.1)
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(v: [f32; 2]) -> Self {
        Self::new(v[0], v[1])
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<f32> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x - rhs, self.y - rhs)
    }
}

impl Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Div for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<f32> for Vec2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<f32> for Vec2 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
/// A struct representing a 3-dimensional vector, with `x`, `y`, and `z` components.
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// A [`Vec3`] with all values set to 0.
    pub const ZERO: Vec3 = Vec3::new(0., 0., 0.);
    /// A [`Vec3`] with all values set to 1.
    pub const ONE:  Vec3 = Vec3::new(1., 1., 1.);

    /// Create a new [`Vec3`] with `x`, `y`, and `z`.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Get the length of this [`Vec3`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Get a normalized copy of this [`Vec3`].
    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }

    /// Get the spherical angles of this [`Vec3`] in radians.
    pub fn angles_rad(&self) -> Vec2 {
        Vec2::new(
            Vec2::new(self.x, self.y).angle_rad(),
            (self.z / self.length()).acos(),
        )
    }

    /// Get the spherical angles of this [`Vec3`] in degrees.
    pub fn angles_deg(&self) -> Vec2 {
        Vec2::new(
            Vec2::new(self.x, self.y).angle_deg(),
            (self.z / self.length()).acos().to_degrees(),
        )
    }

    /// Get the distance between this [`Vec3`] and `other`.
    pub fn dist(&self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl From<Vec3> for (f32, f32, f32) {
    fn from(v: Vec3) -> Self {
        (v.x, v.y, v.z)
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(v: (f32, f32, f32)) -> Self {
        Self::new(v.0, v.1, v.2)
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(v: [f32; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<f32> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<f32> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    /// A [`Vec4`] with all values set to 0.
    pub const ZERO: Vec4 = Vec4::new(0., 0., 0., 0.);
    /// A [`Vec4`] with all values set to 1.
    pub const ONE:  Vec4 = Vec4::new(1., 1., 1., 1.);

    /// Create a new [`Vec4`] with `x`, `y`, `z`, and `w`.
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Get the length of this [`Vec4`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Get a normalized copy of this [`Vec4`].
    pub fn normalize(&self) -> Vec4 {
        let len = self.length();
        Vec4::new(self.x / len, self.y / len, self.z / len, self.w / len)
    }

    /// Get the hyperspherical angles of this [`Vec4`] in radians.
    pub fn angles_rad(&self) -> Vec3 {
        let inner_angle = Vec3::new(self.x, self.y, self.z).angles_rad();
        Vec3::new(
            (self.w / self.length()).acos(),
            inner_angle.y,
            inner_angle.x,
        )
    }

    /// Get the hyperspherical angles of this [`Vec4`] in degrees.
    pub fn angles_deg(&self) -> Vec3 {
        let inner_angle = Vec3::new(self.x, self.y, self.z).angles_deg();
        Vec3::new(
            (self.w / self.length()).acos().to_degrees(),
            inner_angle.y,
            inner_angle.x,
        )
    }

    /// Get the distance between this [`Vec4`] and `other`.
    pub fn dist(&self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        let dw = other.w - self.w;
        (dx * dx + dy * dy + dz * dz + dw * dw).sqrt()
    }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(v: Vec4) -> Self {
        (v.x, v.y, v.z, v.w)
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(v: (f32, f32, f32, f32)) -> Self {
        Self::new(v.0, v.1, v.2, v.3)
    }
}

impl From<Vec4> for [f32; 4] {
    fn from(v: Vec4) -> Self {
        [v.x, v.y, v.z, v.w]
    }
}

impl From<[f32; 4]> for Vec4 {
    fn from(v: [f32; 4]) -> Self {
        Self::new(v[0], v[1], v[2], v[3])
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Self) -> Self::Output {
        Vec4::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
    }
}

impl Add<f32> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: f32) -> Self::Output {
        Vec4::new(self.x + rhs, self.y + rhs, self.z + rhs, self.w + rhs)
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec4::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
    }
}

impl Sub<f32> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: f32) -> Self::Output {
        Vec4::new(self.x - rhs, self.y - rhs, self.z - rhs, self.w - rhs)
    }
}

impl Mul for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec4::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z, self.w * rhs.w)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Div for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: Self) -> Self::Output {
        Vec4::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z, self.w / rhs.w)
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f32) -> Self::Output {
        Vec4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
        self.w += rhs;
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
        self.w -= rhs;
    }
}

impl MulAssign for Vec4 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl DivAssign for Vec4 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }
}

impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}
