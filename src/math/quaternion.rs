use crate::math::vector::Vector3D;
use crate::math::VectorLike;

const QUATERNION_FLAG_UNIT: u8 =                0b__1;
const QUATERNION_FLAG_POSITIVE_POLAR: u8 =      0b_10; // positive polar
const QUATERNION_FLAG_IDENTITY: u8 =            0b100;
const QUATERNION_FLAG_ALL: u8 = QUATERNION_FLAG_UNIT | QUATERNION_FLAG_POSITIVE_POLAR | QUATERNION_FLAG_IDENTITY;


pub struct Quaternion {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
    flags: u8
}

impl Quaternion {

    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        let mut ret = Self {
            w, x, y, z,
            flags: 0u8
        };
        ret.update_flags();
        ret
    }

    pub fn identity() -> Self {
        return Self {
            w: 1f32,
            x: 0f32,
            y: 0f32,
            z: 0f32,
            flags: QUATERNION_FLAG_ALL
        };
    }

    pub fn from_euler(mut x: f32, mut y: f32, mut z: f32) -> Self {
        x *= 0.5f32;
        y *= 0.5f32;
        z *= 0.5f32;

        let cr: f32 = x.cos();
        let sr: f32 = x.sin();
        let cp: f32 = y.cos();
        let sp: f32 = y.sin();
        let cy: f32 = z.cos();
        let sy: f32 = z.sin();

        return Self {
            w: (cr * cp * cy) + (sr * sp * sy),
            x: (sr * cp * cy) - (cr * sp * sy),
            y: (cr * sp * cy) + (sr * cp * sy),
            z: (cr * cp * sy) - (sr * sp * cy),
            flags: QUATERNION_FLAG_UNIT
        };
    }

    pub fn from_principal(mut angle: f32) -> Self {
        angle *= 0.5f32;
        return Self {
            w: angle.cos(),
            x: 0f32,
            y: angle.sin(),
            z: 0f32,
            flags: QUATERNION_FLAG_UNIT
        };
    }

    pub fn copy(&self) -> Self {
        return Self {
            w: self.w,
            x: self.w,
            y: self.y,
            z: self.z,
            flags: self.flags
        };
    }

    fn check_flag(&self, flag: u8) -> bool {
        return (self.flags & flag) != 0u8;
    }

    fn update_flags(&mut self) {
        if self.check_flag(QUATERNION_FLAG_IDENTITY) { return; }
        let identity: Quaternion = Quaternion::identity();
        if self.fuzzy_equals(&identity) {
            self.flags = identity.flags;
            return;
        }

        if self.check_flag(QUATERNION_FLAG_UNIT) { return; }
        if (self.norm_sqr() - 1f32).abs() <= f32::EPSILON {
            self.flags |= QUATERNION_FLAG_UNIT;
        }
    }

    pub fn make_positive_polar(&mut self) {
        if self.check_flag(QUATERNION_FLAG_POSITIVE_POLAR) { return; }
        self.normalize();
        if self.w < 0f32 {
            self.negate();
        }
        self.flags |= QUATERNION_FLAG_POSITIVE_POLAR;
    }

    pub fn scalar_part(&self) -> f32 {
        return self.w;
    }

    pub fn vector_part(&self) -> Vector3D {
        return Vector3D::new(self.x, self.y, self.z);
    }

    pub fn invert(&mut self) {
        if self.check_flag(QUATERNION_FLAG_IDENTITY) { return; }
        let mut norm: f32 = self.norm_sqr();
        if norm > f32::EPSILON {
            norm = 1f32 / norm.sqrt();
            self.w = self.w * norm;
            self.x = -self.x * norm;
            self.y = -self.y * norm;
            self.z = -self.z * norm;
            self.flags &= 0b11;
        }
        // invalid
    }

    pub fn rotate(&self, v: &Vector3D) -> Vector3D {
        if self.check_flag(QUATERNION_FLAG_IDENTITY) { return v.copy(); }

        /*
        let mut u: Vector3D = self.vector_part();
        let dot_uv: f32 = u.dot(v);
        let dot_uu: f32 = u.norm_sqr();
        let mut cross: Vector3D = u.cross(v);

        let mut ret = v.copy();
        ret.multiply_scalar(self.w * self.w - dot_uu);

        u.multiply_scalar(dot_uv * 2f32);
        ret += u;

        cross.multiply_scalar(self.w * 1f32);
        ret += cross;

        return ret;
         */
        return Vector3D {
            x:  self.w * self.w * v.x + 2f32 * self.y * self.w * v.z - 2f32 * self.z * self.w * v.y + self.x * self.x
                * v.x + 2f32 * self.y * self.x * v.y + 2f32 * self.z * self.x * v.z - self.z * self.z * v.x - self.y
                * self.y * v.x,
            y: 2f32 * self.x * self.y * v.x + self.y * self.y * v.y + 2f32 * self.z * self.y * v.z + 2f32 * self.w
                * self.z * v.x - self.z * self.z * v.y + self.w * self.w * v.y - 2f32 * self.x * self.w * v.z - self.x
                * self.x * v.y,
            z: 2f32 * self.x * self.z * v.x + 2f32 * self.y * self.z * v.y + self.z * self.z * v.z - 2f32 * self.w
                * self.y * v.x - self.y * self.y * v.z + 2f32 * self.w * self.x * v.y - self.x * self.x * v.z + self.w
                * self.w * v.z
        };
    }

    pub fn slerp(&self, other: &Quaternion, mut t: f32) -> Quaternion {
        let mut a: Quaternion = self.copy();
        a.make_positive_polar();
        let mut b: Quaternion = other.copy();
        b.make_positive_polar();

        if t.abs() <= f32::EPSILON { return a; }
        let mut j: f32 = 1f32 - t;
        if j.abs() <= f32::EPSILON { return b; }

        let mut dot: f32 = a.dot(&b);
        if dot < 0f32 {
            dot = -dot;
            b.negate();
        }

        if dot <= 0.9995f32 {
            let theta: f32 = dot.acos();
            let sin_theta: f32 = theta.sin();

            j = (j * theta) / sin_theta;
            t = (t * theta) / sin_theta;
        }

        let mut ret: Quaternion = Quaternion {
            w: (j * a.w) + (t * b.w),
            x: (j * a.x) + (t * b.x),
            y: (j * a.y) + (t * b.y),
            z: (j * a.z) + (t * b.z),
            flags: 0
        };
        ret.make_positive_polar();
        ret
    }

    pub fn multiply(&mut self, other: &Quaternion) {
        if self.check_flag(QUATERNION_FLAG_IDENTITY) {
            *self = other.copy();
            return;
        }

        let nx: f32 = self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y;
        let ny: f32 = self.w * other.y + self.y * other.w + self.z * other.x - self.x * other.z;
        let nz: f32 = self.w * other.z + self.z * other.w + self.x * other.y - self.y * other.x;

        self.w = self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z;
        self.x = nx;
        self.y = ny;
        self.z = nz;
        self.flags &= other.flags & QUATERNION_FLAG_UNIT;
    }

}

impl VectorLike for Quaternion {

    fn get_components(&self) -> Vec<f32> {
        return vec![self.w, self.x, self.y, self.z];
    }

    fn get_dimensions(&self) -> usize {
        4usize
    }

    fn set_components(&mut self, c: &[f32]) {
        let clen: usize = c.len();
        if clen < 4usize {
            if clen < 1usize { return; }
            self.w = c[0];
            if clen < 2usize { return; }
            self.x = c[1];
            if clen < 3usize { return; }
            self.y = c[2];
        } else {
            self.w = c[0];
            self.x = c[1];
            self.y = c[2];
            self.z = c[3];
        }
        self.flags = 0u8;
        self.update_flags();
    }

    fn norm_sqr(&self) -> f32 {
        if self.check_flag(QUATERNION_FLAG_UNIT) { return 1f32; }
        return (self.w * self.w) + (self.x * self.x) + (self.y * self.y) + (self.z * self.z);
    }

    fn norm(&self) -> f32 {
        if self.check_flag(QUATERNION_FLAG_UNIT) { return 1f32; }
        return self.norm_sqr().sqrt();
    }

    fn normalize(&mut self) {
        if self.check_flag(QUATERNION_FLAG_UNIT) { return; }
        let norm: f32 = self.norm();
        if norm < f32::EPSILON {
            return;
        }
        self.w /= norm;
        self.x /= norm;
        self.y /= norm;
        self.z /= norm;
        self.flags |= QUATERNION_FLAG_UNIT;
    }

    fn negate(&mut self) {
        self.w = -self.w;
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self.flags &= QUATERNION_FLAG_UNIT;
    }

}
