use crate::math::transform::Transform;
use crate::math::vector::Vector3D;
use std::f32::consts::PI;
use crate::math::quad::Quad;

pub struct Camera {
    pub transform: Transform,

    fov: f32,
    z_near: f32,
    z_far: f32,
    width: f32,
    height: f32,

    fov_tan: f32,
    max_dim: f32,
    pad_left: f32,
    pad_top: f32,
    near_span_half: f32,
    near_span: f32,
}

impl Camera {

    pub fn new() -> Self {
        return Camera {
            transform: Transform::new(),

            fov: 90f32,
            z_near: 0.05f32,
            z_far: 100f32,
            width: 512f32,
            height: 512f32,

            fov_tan: 1f32,
            max_dim: 512f32,
            pad_left: 0f32,
            pad_top: 0f32,
            near_span_half: 0.05f32,
            near_span: 0.1f32
        }
    }

    pub fn get_fov(&self) -> f32 {
        return self.fov;
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.fov_tan = (PI / 360f32) * fov;
        self.update_z_near_line();
    }

    pub fn get_z_near(&self) -> f32 {
        return self.z_near;
    }

    pub fn get_z_far(&self) -> f32 {
        return self.z_far;
    }

    pub fn set_z_near(&mut self, near: f32) {
        self.set_z_planes(near, self.z_far);
    }

    pub fn set_z_far(&mut self, far: f32) {
        self.set_z_planes(self.z_near, far);
    }

    pub fn set_z_planes(&mut self, mut near: f32, mut far: f32) {
        if near < f32::EPSILON { near = f32::EPSILON; }
        if far <= near { far = near + f32::EPSILON; }
        self.z_near = near;
        self.z_far = far;
        self.update_z_near_line();
    }

    fn update_z_near_line(&mut self) {
        self.near_span_half = self.z_near * self.fov_tan;
        self.near_span = self.near_span_half * 2f32;
    }

    pub fn get_width(&self) -> f32 {
        return self.width;
    }

    pub fn get_height(&self) -> f32 {
        return self.height;
    }

    pub fn set_width(&mut self, w: f32) {
        self.set_size(w, self.height);
    }

    pub fn set_height(&mut self, h: f32) {
        self.set_size(self.width, h);
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        let max: f32;
        let pl: f32;
        let pt: f32;
        if w >= h {
            max = w;
            pl = 0f32;
            pt = (w - h) / 2f32;
        } else {
            max = h;
            pl = (h - w) / 2f32;
            pt = 0f32;
        }
        self.max_dim = max;
        self.pad_left = pl;
        self.pad_top = pt;
    }

    pub fn project_point(&self, vec: &mut Vector3D) -> bool {
        self.transform.inverse_transform_vector(vec);
        if !self.project_single(&mut vec.x, vec.z) { return false; }
        if !self.project_single(&mut vec.y, vec.z) { return false; }
        vec.x = vec.x * self.max_dim + self.pad_left;
        vec.y = (1f32 - vec.y) * self.max_dim + self.pad_top;
        return true;
    }

    fn project_single(&self, term: &mut f32, z: f32) -> bool {
        if (*term).abs() < f32::EPSILON {
            *term = 0.5f32;
            return true;
        }
        if z < self.z_near {
            return false;
        }
        let m: f32 = *term / z;
        let ret: f32 = ((m * self.z_near) / self.near_span) + 0.5f32;
        if ret < 0f32 || ret > 1f32 { return false; }
        *term = ret;
        return true;
    }

    pub fn project_quad(&self, quad: &mut Quad) -> bool {
        if !self.project_point(&mut quad.tl) { return false; }
        if !self.project_point(&mut quad.tr) { return false; }
        if !self.project_point(&mut quad.bl) { return false; }
        return self.project_point(&mut quad.br);
    }

}
