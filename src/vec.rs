use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0., 0.);
    pub const ONE:  Vec2 = Vec2::new(1., 1.);
}

impl From<Vec2> for (f32, f32) {
    fn from(value: Vec2) -> Self {
        (value.x, value.y)
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

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        Vec2::new(self.x / len, self.y / len)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0., 0., 0.);
    pub const ONE:  Vec3 = Vec3::new(1., 1., 1.);
}

impl From<Vec3> for (f32, f32, f32) {
    fn from(value: Vec3) -> Self {
        (value.x, value.y, value.z)
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

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(v: [f32; 3]) -> Self {
        Self { x: v[0], y: v[1], z: v[2] }
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
    pub const ZERO: Vec4 = Vec4::new(0., 0., 0., 0.);
    pub const ONE:  Vec4 = Vec4::new(1., 1., 1., 0.);
}

impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(value: Vec4) -> Self {
        (value.x, value.y, value.z, value.w)
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

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(&self) -> Vec4 {
        let len = self.length();
        Vec4::new(self.x / len, self.y / len, self.z / len, self.w / len)
    }
}

impl From<[f32; 4]> for Vec4 {
    fn from(v: [f32; 4]) -> Self {
        Self { x: v[0], y: v[1], z: v[2], w: v[3] }
    }
}
