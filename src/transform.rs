use std::ops::Mul;

use bytemuck::{Pod, Zeroable};

use crate::vec::Vec2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuTransform2d {
    // col0      col1      col2      col3
    // [m11,     [m12,     [m13,     [0,
    //  m21,      m22,      m23,      0,
    //  0,        0,        1,        0,
    //  0]        0]        0]        1]
    pub matrix: [[f32; 4]; 4],
}

impl From<Transform2d> for GpuTransform2d {
    fn from(t: Transform2d) -> Self {
        let [m11, m21, m12, m22, m13, m23] = t.matrix;
        Self {
            matrix: [
                [m11, m21, 0. , 0. ],
                [m12, m22, 0. , 0. ],
                [0. , 0. , 1. , 0. ],
                [m13, m23, 0. , 1. ],
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform2d {
    // represents
    // [ m11 m12 m13 ]
    // [ m21 m22 m23 ]
    // [ 0   0   1   ]
    // with indices
    // [ 0   2   4   ]
    // [ 1   3   5   ]
    // [ N/A N/A N/A ]
    pub matrix: [f32; 6],
}

impl AsRef<Transform2d> for Transform2d {
    fn as_ref(&self) -> &Transform2d {
        self
    }
}

impl Default for Transform2d {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2d {
    /// Returns the identity transform — no translation, rotation, or scale.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::identity();
    /// let p = Vec2::new(3., 4.);
    /// assert_eq!(t.transform_point(p), p);
    /// ```
    pub fn identity() -> Self {
        Self {
            matrix: [
                1., 0.,
                0., 1.,
                0., 0.,
            ]
        }
    }

    /// Applies `self` first, then `other`, returning the composed transform.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::translation(1., 0.).then(Transform2d::scaling(2., 2.));
    /// let p = t.transform_point(Vec2::new(1., 0.));
    /// assert_eq!(p, Vec2::new(4., 0.)); // translated first, then scaled
    /// ```
    pub fn then(self, other: Self) -> Self {
        other.mul(self)
    }

    /// Returns a transform that translates by `(x, y)`.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::translation(5., -3.);
    /// assert_eq!(t.transform_point(Vec2::new(1., 1.)), Vec2::new(6., -2.));
    /// ```
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            matrix: [
                1., 0.,
                0., 1.,
                x,  y,
            ]
        }
    }

    /// Applies an additional translation of `(x, y)` after `self` and returns the result.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let mut t = Transform2d::scaling(2., 2.);
    /// let t2 = t.translate(3., 1.);
    /// assert_eq!(t2.transform_point(Vec2::new(1., 1.)), Vec2::new(5., 3.));
    /// ```
    pub fn translate(&mut self, x: f32, y: f32) -> &mut Self {
        *self = self.then(Self::translation(x, y));
        self
    }

    /// Returns a transform that rotates by `rad` radians, counter-clockwise.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::rotation_rad(std::f32::consts::FRAC_PI_2);
    /// let p = t.transform_point(Vec2::new(1., 0.));
    /// assert!((p.x - 0.).abs() < 1e-6);
    /// assert!((p.y - 1.).abs() < 1e-6);
    /// ```
    pub fn rotation_rad(rad: f32) -> Self {
        let (sin, cos) = rad.sin_cos();
        Self {
            matrix: [
                cos,  sin,
                -sin, cos,
                0.,   0.,
            ]
        }
    }

    /// Applies an additional counter-clockwise rotation of `rad` radians after `self` and returns the result.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let mut t = Transform2d::translation(1., 0.);
    /// let t2 = t.rotate_rad(std::f32::consts::FRAC_PI_2);
    /// let p = t2.transform_point(Vec2::new(0., 0.));
    /// assert!((p.x - 0.).abs() < 1e-6);
    /// assert!((p.y - 1.).abs() < 1e-6);
    /// ```
    pub fn rotate_rad(&mut self, rad: f32) -> &mut Self {
        *self = self.then(Self::rotation_rad(rad));
        self
    }

    /// Returns a transform that rotates by `deg` degrees, counter-clockwise.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::rotation_deg(90.);
    /// let p = t.transform_point(Vec2::new(1., 0.));
    /// assert!((p.x - 0.).abs() < 1e-6);
    /// assert!((p.y - 1.).abs() < 1e-6);
    /// ```
    pub fn rotation_deg(deg: f32) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        Self {
            matrix: [
                cos,  sin,
                -sin, cos,
                0.,   0.,
            ]
        }
    }

    /// Applies an additional counter-clockwise rotation of `deg` degrees after `self` and returns the result.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let mut t = Transform2d::translation(1., 0.);
    /// let t2 = t.rotate_deg(90.);
    /// let p = t2.transform_point(Vec2::new(0., 0.));
    /// assert!((p.x - 0.).abs() < 1e-6);
    /// assert!((p.y - 1.).abs() < 1e-6);
    /// ```
    pub fn rotate_deg(&mut self, deg: f32) -> &mut Self {
        *self = self.then(Self::rotation_deg(deg));
        self
    }

    /// Returns a transform that scales by `sx` horizontally and `sy` vertically.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::scaling(2., 3.);
    /// assert_eq!(t.transform_point(Vec2::new(1., 1.)), Vec2::new(2., 3.));
    /// ```
    pub fn scaling(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [
                sx, 0.,
                0., sy,
                0., 0.,
            ]
        }
    }

    /// Applies an additional scale of `sx` horizontally and `sy` vertically after `self` and returns the result.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::translation(1., 1.).scale(2., 3.);
    /// assert_eq!(t.transform_point(Vec2::new(0., 0.)), Vec2::new(2., 3.));
    /// ```
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        *self = self.then(Transform2d::scaling(sx, sy));
        self
    }

    /// Applies this transform to a 2D point.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::translation(1., 2.);
    /// assert_eq!(t.transform_point(Vec2::new(3., 4.)), Vec2::new(4., 6.));
    /// ```
    pub fn transform_point(self, p: Vec2) -> Vec2 {
        let [m11, m21, m12, m22, m13, m23] = self.matrix;
        Vec2::new(
            m11 * p.x + m12 * p.y + m13,
            m21 * p.x + m22 * p.y + m23,
        )
    }

    /// Returns the scale factors encoded in this transform as a [`Vec2`],
    /// extracted from the column magnitudes of the rotation/scale portion of the matrix.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    ///
    /// let t = Transform2d::scaling(2., 3.).then(Transform2d::rotation_deg(45.));
    /// let s = t.get_scale();
    /// assert!((s.x - 2.).abs() < 1e-6);
    /// assert!((s.y - 3.).abs() < 1e-6);
    /// ```
    pub fn get_scale(self) -> Vec2 {
        Vec2::new(
            (self.matrix[0] * self.matrix[0] + self.matrix[1] * self.matrix[1]).sqrt(),
            (self.matrix[2] * self.matrix[2] + self.matrix[3] * self.matrix[3]).sqrt(),
        )
    }
}

impl Mul for Transform2d {
    type Output = Self;

    /// Multiplies two transforms together, composing them into a single transform.
    /// The result applies `other` first, then `self`.
    ///
    /// # Example
    /// ```
    /// use verdant::{transform::Transform2d, vec::Vec2};
    /// use std::ops::Mul;
    ///
    /// let t = Transform2d::translation(2., 0.).mul(Transform2d::scaling(2., 2.));
    /// let p = t.transform_point(Vec2::new(1., 0.));
    /// assert_eq!(p, Vec2::new(4., 0.)); // scaled first, then translated
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let [lm11, lm21, lm12, lm22, lm13, lm23] = self.matrix;
        let [rm11, rm21, rm12, rm22, rm13, rm23] = rhs.matrix;

        let m11 = lm11 * rm11 + lm12 * rm21;
        let m21 = lm21 * rm11 + lm22 * rm21;
        let m12 = lm11 * rm12 + lm12 * rm22;
        let m22 = lm21 * rm12 + lm22 * rm22;
        let m13 = lm11 * rm13 + lm12 * rm23 + lm13; // translation x
        let m23 = lm21 * rm13 + lm22 * rm23 + lm23; // translation y

        Self {
            matrix: [
                m11, m21,
                m12, m22,
                m13, m23,
            ]
        }
    }
}
