use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};

// lmao
// it's practical okay
macro_rules! op_binop {
    ($t:ty, $opname:ident, $func:ident, $op:tt; $($var:ident),*) => {
        impl $opname for $t {
            type Output = Self;
            fn $func(self, rhs: Self) -> Self::Output {
                Self::new($(self.$var $op rhs.$var),*)
            }
        }
    };
    ($t:ty, $opname:ident, $func:ident, $op:tt, $rhs:ty; $($var:ident),*) => {
        impl $opname<$rhs> for $t {
            type Output = Self;
            fn $func(self, rhs: $rhs) -> Self::Output {
                Self::new($(self.$var $op rhs.$var),*)
            }
        }
    };
    ($t:ty, $opname:ident, $func:ident, $op:tt, $rhs:ty [scalar]; $($var:ident),*) => {
        impl $opname<$rhs> for $t {
            type Output = Self;
            fn $func(self, rhs: $rhs) -> Self::Output {
                Self::new($(self.$var $op rhs),*)
            }
        }
    };
}

macro_rules! op_assign {
    ($t:ty, $opname:ident, $func:ident, $op:tt; $($var:ident),*) => {
        impl $opname for $t {
            fn $func(&mut self, rhs: Self) {
                $(self.$var $op rhs.$var;)*
            }
        }
    };
    ($t:ty, $opname:ident, $func:ident, $op:tt, $rhs:ty; $($var:ident),*) => {
        impl $opname<$rhs> for $t {
            fn $func(&mut self, rhs: $rhs) {
                $(self.$var $op rhs.$var;)*
            }
        }
    };
    ($t:ty, $opname:ident, $func:ident, $op:tt, $rhs:ty [scalar]; $($var:ident),*) => {
        impl $opname<$rhs> for $t {
            fn $func(&mut self, rhs: $rhs) {
                $(self.$var $op rhs;)*
            }
        }
    };
}

macro_rules! vec_ops {
    ($t:ty; $($var:ident),*) => {
        op_binop!($t, Add, add, +; $($var),*);
        op_binop!($t, Sub, sub, -; $($var),*);
        op_binop!($t, Mul, mul, *; $($var),*);
        op_binop!($t, Div, div, /; $($var),*);

        op_assign!($t, AddAssign, add_assign, +=; $($var),*);
        op_assign!($t, SubAssign, sub_assign, -=; $($var),*);
        op_assign!($t, MulAssign, mul_assign, *=; $($var),*);
        op_assign!($t, DivAssign, div_assign, /=; $($var),*);
    };
    ($t:ty, $rhs:ty; $($var:ident),*) => {
        op_binop!($t, Add, add, +, $rhs; $($var),*);
        op_binop!($t, Sub, sub, -, $rhs; $($var),*);
        op_binop!($t, Mul, mul, *, $rhs; $($var),*);
        op_binop!($t, Div, div, /, $rhs; $($var),*);

        op_assign!($t, AddAssign, add_assign, +=, $rhs; $($var),*);
        op_assign!($t, SubAssign, sub_assign, -=, $rhs; $($var),*);
        op_assign!($t, MulAssign, mul_assign, *=, $rhs; $($var),*);
        op_assign!($t, DivAssign, div_assign, /=, $rhs; $($var),*);
    };
    ($t:ty, $rhs:ty [scalar]; $($var:ident),*) => {
        op_binop!($t, Add, add, +, $rhs [scalar]; $($var),*);
        op_binop!($t, Sub, sub, -, $rhs [scalar]; $($var),*);
        op_binop!($t, Mul, mul, *, $rhs [scalar]; $($var),*);
        op_binop!($t, Div, div, /, $rhs [scalar]; $($var),*);

        op_assign!($t, AddAssign, add_assign, +=, $rhs [scalar]; $($var),*);
        op_assign!($t, SubAssign, sub_assign, -=, $rhs [scalar]; $($var),*);
        op_assign!($t, MulAssign, mul_assign, *=, $rhs [scalar]; $($var),*);
        op_assign!($t, DivAssign, div_assign, /=, $rhs [scalar]; $($var),*);
    };
}

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

    /// Returns the length of this [`Vec2`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns the squared length of this [`Vec2`]
    /// Useful for comparing lengths of vectors without needing a `sqrt`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns a normalized copy of this [`Vec2`].
    pub fn normalize(&self) -> Vec2 {
        *self / self.length()
    }

    /// Returns the angle of this [`Vec2`] in radians.
    pub fn angle_rad(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Returns the angle of this [`Vec2`] in degrees.
    pub fn angle_deg(&self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }

    /// Returns the distance between this [`Vec2`] and `other`.
    pub fn dist(&self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the component-wise maximum of this [`Vec2`] and `other`.
    pub fn max(&self, other: Self) -> Vec2 {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    /// Returns the longer vector between this [`Vec2`] and `other`.
    pub fn longest(&self, other: Self) -> Vec2 {
        if self.length_squared() >= other.length_squared() {
            *self
        } else {
            other
        }
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

vec_ops!(Vec2; x, y);
vec_ops!(Vec2, f32 [scalar]; x, y);

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

    /// Returns the length of this [`Vec3`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the squared length of this [`Vec3`]
    /// Useful for comparing lengths of vectors without needing a `sqrt`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns a normalized copy of this [`Vec3`].
    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }

    /// Returns the spherical angles of this [`Vec3`] in radians.
    pub fn angles_rad(&self) -> Vec2 {
        Vec2::new(
            Vec2::new(self.x, self.y).angle_rad(),
            (self.z / self.length()).acos(),
        )
    }

    /// Returns the spherical angles of this [`Vec3`] in degrees.
    pub fn angles_deg(&self) -> Vec2 {
        Vec2::new(
            Vec2::new(self.x, self.y).angle_deg(),
            (self.z / self.length()).acos().to_degrees(),
        )
    }

    /// Returns the distance between this [`Vec3`] and `other`.
    pub fn dist(&self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Returns the component-wise maximum of this [`Vec3`] and `other`.
    pub fn max(&self, other: Self) -> Vec3 {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    /// Returns the longer vector between this [`Vec3`] and `other`.
    pub fn longest(&self, other: Self) -> Vec3 {
        if self.length_squared() >= other.length_squared() {
            *self
        } else {
            other
        }
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

vec_ops!(Vec3; x, y, z);
vec_ops!(Vec3, f32 [scalar]; x, y, z);

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

    /// Returns the length of this [`Vec4`].
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Returns the squared length of this [`Vec4`]
    /// Useful for comparing lengths of vectors without needing a `sqrt`.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Returns a normalized copy of this [`Vec4`].
    pub fn normalize(&self) -> Vec4 {
        let len = self.length();
        Vec4::new(self.x / len, self.y / len, self.z / len, self.w / len)
    }

    /// Returns the hyperspherical angles of this [`Vec4`] in radians.
    pub fn angles_rad(&self) -> Vec3 {
        let inner_angle = Vec3::new(self.x, self.y, self.z).angles_rad();
        Vec3::new(
            (self.w / self.length()).acos(),
            inner_angle.y,
            inner_angle.x,
        )
    }

    /// Returns the hyperspherical angles of this [`Vec4`] in degrees.
    pub fn angles_deg(&self) -> Vec3 {
        let inner_angle = Vec3::new(self.x, self.y, self.z).angles_deg();
        Vec3::new(
            (self.w / self.length()).acos().to_degrees(),
            inner_angle.y,
            inner_angle.x,
        )
    }

    /// Returns the distance between this [`Vec4`] and `other`.
    pub fn dist(&self, other: impl Into<Self>) -> f32 {
        let other = other.into();
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        let dw = other.w - self.w;
        (dx * dx + dy * dy + dz * dz + dw * dw).sqrt()
    }

    /// Returns the component-wise maximum of this [`Vec4`] and `other`.
    pub fn max(&self, other: impl Into<Self>) -> Vec4 {
        let other = other.into();
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }

    /// Returns the longer vector between this [`Vec4`] and `other`.
    pub fn longest(&self, other: impl Into<Self>) -> Vec4 {
        let other = other.into();
        if self.length_squared() >= other.length_squared() {
            *self
        } else {
            other
        }
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

vec_ops!(Vec4; x, y, z, w);
vec_ops!(Vec4, f32 [scalar]; x, y, z, w);

#[cfg(feature = "glam")]
mod glam_compat {
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

    use super::{Vec2, Vec3, Vec4};

    impl From<glam::Vec2> for Vec2 {
        fn from(v: glam::Vec2) -> Self {
            Self::new(v.x, v.y)
        }
    }

    impl From<Vec2> for glam::Vec2 {
        fn from(v: Vec2) -> Self {
            Self::new(v.x, v.y)
        }
    }

    vec_ops!(Vec2, glam::Vec2; x, y);
    vec_ops!(glam::Vec2, Vec2; x, y);

    impl From<glam::Vec3> for Vec3 {
        fn from(v: glam::Vec3) -> Self {
            Self::new(v.x, v.y, v.z)
        }
    }

    impl From<Vec3> for glam::Vec3 {
        fn from(v: Vec3) -> Self {
            Self::new(v.x, v.y, v.z)
        }
    }

    vec_ops!(Vec3, glam::Vec3; x, y, z);
    vec_ops!(glam::Vec3, Vec3; x, y, z);

    impl From<glam::Vec4> for Vec4 {
        fn from(v: glam::Vec4) -> Self {
            Self::new(v.x, v.y, v.z, v.w)
        }
    }

    impl From<Vec4> for glam::Vec4 {
        fn from(v: Vec4) -> Self {
            Self::new(v.x, v.y, v.z, v.w)
        }
    }

    vec_ops!(Vec4, glam::Vec4; x, y, z, w);
    vec_ops!(glam::Vec4, Vec4; x, y, z, w);
}
