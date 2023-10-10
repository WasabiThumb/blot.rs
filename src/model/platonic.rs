use crate::math::quad::Quad;
use crate::math::vector::Vector3D;
use crate::model::Model;

const CUBE_FACES: [Quad; 6] = [
    Quad {
        tl: Vector3D { x: -1.0, y: 1.0, z: 1.0 },
        tr: Vector3D { x: 1.0, y: 1.0, z: 1.0 },
        bl: Vector3D { x: -1.0, y: -1.0, z: 1.0 },
        br: Vector3D { x: 1.0, y: -1.0, z: 1.0 },
    },
    Quad {
        tl: Vector3D { x: -1.0, y: 1.0, z: -1.0 },
        tr: Vector3D { x: 1.0, y: 1.0, z: -1.0 },
        bl: Vector3D { x: -1.0, y: -1.0, z: -1.0 },
        br: Vector3D { x: 1.0, y: -1.0, z: -1.0 },
    },
    Quad {
        tl: Vector3D { x: -1.0, y: 1.0, z: 1.0 },
        tr: Vector3D { x: -1.0, y: 1.0, z: -1.0 },
        bl: Vector3D { x: -1.0, y: -1.0, z: 1.0 },
        br: Vector3D { x: -1.0, y: -1.0, z: -1.0 },
    },
    Quad {
        tl: Vector3D { x: 1.0, y: 1.0, z: -1.0 },
        tr: Vector3D { x: 1.0, y: 1.0, z: 1.0 },
        bl: Vector3D { x: 1.0, y: -1.0, z: -1.0 },
        br: Vector3D { x: 1.0, y: -1.0, z: 1.0 },
    },
    Quad {
        tl: Vector3D { x: -1.0, y: 1.0, z: 1.0 },
        tr: Vector3D { x: 1.0, y: 1.0, z: 1.0 },
        bl: Vector3D { x: -1.0, y: 1.0, z: -1.0 },
        br: Vector3D { x: 1.0, y: 1.0, z: -1.0 },
    },
    Quad {
        tl: Vector3D { x: -1.0, y: -1.0, z: -1.0 },
        tr: Vector3D { x: 1.0, y: -1.0, z: -1.0 },
        bl: Vector3D { x: -1.0, y: -1.0, z: 1.0 },
        br: Vector3D { x: 1.0, y: -1.0, z: 1.0 },
    }
];

pub fn cube() -> Model {
    let mut model: Model = Model::expect(6usize);

    for idx in 0 .. 6usize {
        model.add_face(CUBE_FACES[idx].copy());
    }

    model
}
