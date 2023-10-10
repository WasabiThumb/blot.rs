pub mod platonic;
pub mod sphere;

use crate::math::quad::Quad;
use crate::math::transform::Transform;

struct ModelFace {
    quad: Quad,
    u: [f32; 4],
    v: [f32; 4]
}

pub struct Model {
    pub transform: Transform,
    faces: Vec<ModelFace>,
}

impl Model {

    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
            faces: Vec::new()
        }
    }

    pub fn expect(size: usize) -> Self {
        Self {
            transform: Transform::new(),
            faces: Vec::with_capacity(size)
        }
    }

    pub fn add_face(&mut self, quad: Quad) {
        self.faces.push(ModelFace {
            quad, u: [ 0.0, 1.0, 0.0, 1.0 ], v: [ 0.0, 0.0, 1.0, 1.0 ]
        });
    }

    pub fn add_face_uv(&mut self, quad: Quad, u_tl: f32, v_tl: f32, u_tr: f32, v_tr: f32, u_bl: f32, v_bl: f32, u_br: f32, v_br: f32) {
        self.faces.push(ModelFace {
            quad, u: [ u_tl, u_tr, u_bl, u_br ], v: [ v_tl, v_tr, v_bl, v_br ]
        });
    }

    pub fn get_face_count(&self) -> usize {
        return self.faces.len();
    }

    fn index_check(&self, idx: usize) {
        if idx >= self.faces.len() {
            panic!("Index {} out of bounds for length {}", idx, self.faces.len());
        }
    }

    pub fn get_face(&self, idx: usize) -> Quad {
        self.index_check(idx);
        let mut ret: Quad = self.faces[idx].quad.copy();
        self.transform.transform_quad(&mut ret);
        return ret;
    }

    pub fn remap_face_uv(&self, idx: usize, u: &mut f32, v: &mut f32) {
        self.index_check(idx);

        let face: &ModelFace = &self.faces[idx];
        let us: &[f32; 4] = &face.u;
        let vs: &[f32; 4] = &face.v;

        let u_t: f32 = (us[1] - us[0]) * (*u) + us[0];
        let u_b: f32 = (us[3] - us[2]) * (*u) + us[2];
        let nu: f32 = (u_b - u_t) * (*v) + u_t;

        let v_l: f32 = (vs[2] - vs[0]) * (*v) + vs[0];
        let v_r: f32 = (vs[3] - vs[1]) * (*v) + vs[1];
        *v = (v_r - v_l) * (*u) + v_l;
        *u = nu;
    }

}
