use crate::model::Model;
use core::f32::consts::PI;
use crate::math::quad::Quad;
use crate::math::vector::Vector3D;

pub fn uv_sphere(mut lat: u16, mut long: u16) -> Model {
    if lat < 3u16 { lat = 3u16; }
    if long < 3u16 { long = 3u16; }

    let mut ret: Model = Model::expect((lat * long) as usize);

    let mut ang_pc_start: f32 = 0f32;
    let mut x1: f32 = 1f32;
    let mut z1: f32 = 0f32;
    for ang_step in 1..=lat {
        let ang_pc_end: f32 = (ang_step as f32) / (lat as f32);
        let ang_end: f32 = ang_pc_end * PI * 2f32;

        let x2: f32 = ang_end.cos();
        let z2: f32 = ang_end.sin();

        let mut gamma_pc_start: f32 = 0f32;
        let mut y1: f32 = 1f32;
        let mut f1: f32 = 0f32;
        for gamma_step in 1..=long {
            let gamma_pc_end: f32 = (gamma_step as f32) / (long as f32);
            let gamma_end: f32 = gamma_pc_end * PI;

            let y2: f32 = gamma_end.cos();
            let f2: f32 = (1f32 - y2.powi(2i32)).sqrt();

            ret.add_face_uv(
                Quad {
                    tl: Vector3D::new( x1 * f1, y1, z1 * f1 ),
                    tr: Vector3D::new( x2 * f1, y1, z2 * f1 ),
                    bl: Vector3D::new( x1 * f2, y2, z1 * f2 ),
                    br: Vector3D::new( x2 * f2, y2, z2 * f2 )
                },
                ang_pc_start, gamma_pc_start,
                ang_pc_end, gamma_pc_start,
                ang_pc_start, gamma_pc_end,
                ang_pc_end, gamma_pc_end
            );

            gamma_pc_start = gamma_pc_end;
            y1 = y2;
            f1 = f2;
        }

        ang_pc_start = ang_pc_end;
        x1 = x2;
        z1 = z2;
    }

    ret
}
