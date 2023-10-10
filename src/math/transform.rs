use crate::math::quad::Quad;
use crate::math::quaternion::Quaternion;
use crate::math::vector::Vector3D;
use crate::math::VectorLike;

pub struct Transform {
    pub translation: Vector3D,
    pub rotation: Quaternion,
    pub scale: Vector3D
}

impl Transform {

    pub fn new() -> Self {
        return Transform {
            translation: Vector3D::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3D::uniform(1f32)
        }
    }

    #[allow(dead_code)]
    fn copy(&self) -> Self {
        return Self {
            translation: self.translation.copy(),
            rotation: self.rotation.copy(),
            scale: self.scale.copy()
        }
    }

    pub fn translate(&mut self, amt: &Vector3D) {
        self.translation.add_vector(amt);
    }

    pub fn rotate(&mut self, amt: &Quaternion) {
        self.rotation.multiply(amt);
    }

    pub fn scale(&mut self, amt: &Vector3D) {
        self.scale.multiply_vector(amt);
    }

    pub fn scale_scalar(&mut self, amt: f32) {
        self.scale.multiply_scalar(amt);
    }

    pub fn transform(&self, other: &mut Transform) {
        other.translate(&self.translation);
        other.rotate(&self.rotation);
        other.scale(&self.scale);
    }

    pub fn inverse_transform(&self, other: &mut Transform) {
        let mut inv_translation: Vector3D = self.translation.copy();
        inv_translation.negate();
        other.translate(&inv_translation);

        let mut inv_rotation: Quaternion = self.rotation.copy();
        inv_rotation.invert();
        other.rotate(&inv_rotation);

        other.scale.divide_vector(&self.scale);
    }

    pub fn transform_vector(&self, vec: &mut Vector3D) {
        *vec = self.rotation.rotate(vec);
        vec.multiply_vector(&self.scale);
        vec.add_vector(&self.translation);
    }

    pub fn inverse_transform_vector(&self, vec: &mut Vector3D) {
        vec.subtract_vector(&self.translation);
        vec.divide_vector(&self.scale);

        let mut inv_rotation: Quaternion = self.rotation.copy();
        inv_rotation.invert();
        *vec = inv_rotation.rotate(vec);
    }

    pub fn transform_quad(&self, quad: &mut Quad) {
        self.transform_vector(&mut quad.tl);
        self.transform_vector(&mut quad.tr);
        self.transform_vector(&mut quad.bl);
        self.transform_vector(&mut quad.br);
    }

    pub fn inverse_transform_quad(&self, quad: &mut Quad) {
        self.inverse_transform_vector(&mut quad.tl);
        self.inverse_transform_vector(&mut quad.tr);
        self.inverse_transform_vector(&mut quad.bl);
        self.inverse_transform_vector(&mut quad.br);
    }

}
