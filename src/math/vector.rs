use std::fmt::{Display, Formatter};
use crate::math::VectorLike;
use std::ops;

pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3D {

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { x, y, z };
    }

    pub fn zero() -> Self {
        return Self { x: 0f32, y: 0f32, z: 0f32 };
    }

    pub fn uniform(v: f32) -> Self {
        return Self { x: v, y: v, z: v };
    }

    pub fn lerp(a: &Vector3D, b: &Vector3D, t: f32) -> Self {
        let v: f32 = 1f32 - t;
        return Self {
            x: (b.x * t) + (a.x * v),
            y: (b.y * t) + (a.y * v),
            z: (b.z * t) + (a.z * v)
        };
    }

    pub fn cross(&self, o: &Vector3D) -> Self {
        return Self {
            x: (self.y * o.z) - (self.z * o.y),
            y: (self.z * o.x) - (self.x * o.z),
            z: (self.x * o.y) - (self.y * o.x)
        }
    }

    pub fn copy(&self) -> Self {
        return Self {
            x: self.x,
            y: self.y,
            z: self.z
        };
    }

}

impl VectorLike for Vector3D {

    fn get_components(&self) -> Vec<f32> {
        return vec![ self.x, self.y, self.z ];
    }

    fn get_dimensions(&self) -> usize {
        3usize
    }

    fn set_components(&mut self, c: &[f32]) {
        let clen: usize = c.len();
        if clen < 3 {
            if clen < 1 { return; }
            self.x = c[0];
            if clen < 2 { return; }
            self.y = c[1];
        } else {
            self.x = c[0];
            self.y = c[1];
            self.z = c[2];
        }
    }

    fn norm_sqr(&self) -> f32 {
        return (self.x * self.x) + (self.y * self.y) + (self.z * self.z);
    }

    fn normalize(&mut self) {
        let mut norm: f32 = self.norm_sqr();
        if norm <= f32::EPSILON || (norm - 1f32).abs() <= f32::EPSILON {
            return;
        }
        norm = norm.sqrt();
        self.x /= norm;
        self.y /= norm;
        self.z /= norm;
    }

}

impl ops::Add<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn add(self, rhs: Vector3D) -> Self::Output {
        let mut ret = self.copy();
        ret.add_vector(&rhs);
        return ret;
    }
}

impl ops::AddAssign<Vector3D> for Vector3D {
    fn add_assign(&mut self, rhs: Vector3D) {
        self.add_vector(&rhs);
    }
}

impl ops::Sub<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Vector3D) -> Self::Output {
        let mut ret = self.copy();
        ret.subtract_vector(&rhs);
        return ret;
    }
}

impl ops::SubAssign<Vector3D> for Vector3D {
    fn sub_assign(&mut self, rhs: Vector3D) {
        self.subtract_vector(&rhs);
    }
}

impl ops::Mul<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn mul(self, rhs: Vector3D) -> Self::Output {
        let mut ret = self.copy();
        ret.multiply_vector(&rhs);
        return ret;
    }
}

impl ops::MulAssign<Vector3D> for Vector3D {
    fn mul_assign(&mut self, rhs: Vector3D) {
        self.multiply_vector(&rhs);
    }
}

impl ops::Div<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn div(self, rhs: Vector3D) -> Self::Output {
        let mut ret = self.copy();
        ret.divide_vector(&rhs);
        return ret;
    }
}

impl ops::DivAssign<Vector3D> for Vector3D {
    fn div_assign(&mut self, rhs: Vector3D) {
        self.divide_vector(&rhs);
    }
}

impl PartialEq for Vector3D {

    fn eq(&self, other: &Self) -> bool {
        return (self.x == other.x) && (self.y == other.y) && (self.z == other.z);
    }

}

impl Eq for Vector3D {

}

impl Display for Vector3D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
