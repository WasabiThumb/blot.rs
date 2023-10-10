pub mod quaternion;
pub mod vector;
pub mod line;
pub mod transform;
pub mod quad;


fn vec_update(vec: &mut dyn VectorLike, m: impl Fn(usize, f32) -> f32) {
    let mut components: Vec<f32> = vec.get_components();
    for n in 0 .. components.len() {
        components[n] = m(n, components[n]);
    }
    vec.set_components(&components);
}

fn vec_weave(a: &dyn VectorLike, b: &dyn VectorLike, cb: impl Fn(f32, f32) -> f32) -> f32 {
    let mc: Vec<f32> = a.get_components();
    let md: usize = mc.len();

    let oc: Vec<f32> = b.get_components();
    let od: usize = oc.len();

    let ac: &Vec<f32>;
    let ad: usize;
    let mut ao: usize = 0;
    let nac: &Vec<f32>;
    let nad: usize;

    if md > od {
        if md == 4 {
            ao = 1;
        }
        ac = &mc;
        ad = md;
        nac = &oc;
        nad = od + ao;
    } else {
        if od == 4 && md < 4 {
            ao = 1;
        }
        ac = &oc;
        ad = od;
        nac = &mc;
        nad = md + ao;
    }

    let mut ret: f32 = 0f32;
    for n in 0 .. ad {
        if n >= ao && n < nad {
            ret += cb(ac[n], nac[n - ao]);
        } else {
            ret += cb(ac[n], 0f32);
        }
    }

    ret
}

pub trait VectorLike {
    fn get_components(&self) -> Vec<f32>;
    fn get_dimensions(&self) -> usize;
    fn set_components(&mut self, c: &[f32]);


    fn reduce(&self, m: fn(f32) -> f32, join: fn(f32, f32) -> f32) -> f32 {
        let components: Vec<f32> = self.get_components();
        let clen: usize = components.len();
        if clen < 1 {
            panic!("Attempt to reduce 0-dimensional VectorLike!");
        }

        let mut ret: f32 = m(components[0]);
        for n in 1 .. clen {
            ret = join(ret, m(components[n]));
        }
        return ret;
    }

    fn unary(&mut self, other: &dyn VectorLike, op: fn(f32, f32) -> f32) where Self: Sized {
        let oc: Vec<f32> = other.get_components();
        let mut od: usize = oc.len();
        let oo: usize = if self.get_dimensions() >= 4 && od < 4 {
            od -= 1usize;
            1usize
        } else {
            0usize
        };

        vec_update(self, | idx, v |
            return if idx < od {
                op(v, oc[oo + idx])
            } else {
                v
            }
        );
    }

    fn norm_sqr(&self) -> f32 {
        return self.reduce(
            | c | c * c,
            | a, b | a + b
        );
    }

    fn norm(&self) -> f32 {
        return self.norm_sqr().sqrt();
    }

    fn dot(&self, other: &dyn VectorLike) -> f32 where Self: Sized {
        return vec_weave(self, other, | a, b |
            a * b
        );
    }

    fn add_vector(&mut self, other: &dyn VectorLike) where Self: Sized {
        self.unary(other, | a, b | a + b);
    }

    fn add_scalar(&mut self, other: f32) where Self: Sized {
        vec_update(self, | _, v | v + other);
    }

    fn subtract_vector(&mut self, other: &dyn VectorLike) where Self: Sized {
        self.unary(other, | a, b | a - b);
    }

    fn subtract_scalar(&mut self, other: f32) where Self: Sized {
        vec_update(self, | _, v | v - other);
    }

    fn multiply_vector(&mut self, other: &dyn VectorLike) where Self: Sized {
        self.unary(other, | a, b | a * b);
    }

    fn multiply_scalar(&mut self, other: f32) where Self: Sized {
        vec_update(self, | _, v | v * other);
    }

    fn divide_vector(&mut self, other: &dyn VectorLike) where Self: Sized {
        self.unary(other, | a, b |
            if (b - a).abs() <= f32::EPSILON {
                a
            } else {
                a / b
            }
        );
    }

    fn divide_scalar(&mut self, other: f32) where Self: Sized {
        vec_update(self, | _, v | v / other);
    }

    fn fuzzy_equals(&self, other: &dyn VectorLike) -> bool {
        let mc: Vec<f32> = self.get_components();
        let oc: Vec<f32> = other.get_components();
        let len: usize = mc.len();
        if len != oc.len() {
            return false;
        }
        for n in 0 .. len {
            if (mc[n] - oc[n]).abs() > f32::EPSILON {
                return false;
            }
        }
        return true;
    }

    fn normalize(&mut self) where Self: Sized {
        let norm: f32 = self.norm();
        if norm <= f32::EPSILON {
            return;
        }
        vec_update(self, | _, v | v / norm);
    }

    fn negate(&mut self) where Self: Sized {
        vec_update(self, | _, v | -v);
    }
}
