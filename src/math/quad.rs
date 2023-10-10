use crate::math::vector::Vector3D;
use crate::math::VectorLike;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use crate::math::line::Line;

fn minmax4(a: f32, b: f32, c: f32, d: f32, min: &mut u32, max: &mut u32) {
    let mut min_v: f32 = a;
    let mut max_v: f32 = a;
    if b <= (min_v) { min_v = b; } else if b > (max_v) { max_v = b; }
    if c <= (min_v) { min_v = c; } else if c > (max_v) { max_v = c; }
    if d <= (min_v) { min_v = d; } else if d > (max_v) { max_v = d; }
    *min = min_v.floor() as u32;
    *max = max_v.ceil() as u32;
}

#[derive(Eq, PartialEq)]
pub struct Quad {
    pub tl: Vector3D,
    pub tr: Vector3D,
    pub bl: Vector3D,
    pub br: Vector3D
}

impl Quad {

    pub fn extract_int_bounds(&self, min_x: &mut u32, min_y: &mut u32, max_x: &mut u32, max_y: &mut u32) {
        minmax4(self.tl.x, self.tr.x, self.bl.x, self.br.x, min_x, max_x);
        minmax4(self.tl.y, self.tr.y, self.bl.y, self.br.y, min_y, max_y);
    }

    pub fn copy(&self) -> Self {
        return Self {
            tl: self.tl.copy(),
            tr: self.tr.copy(),
            bl: self.bl.copy(),
            br: self.br.copy()
        };
    }

    pub fn get_u_line(&self, u: f32) -> Line {
        return Line::new(
            &Vector3D::lerp(&self.tl, &self.tr, u),
            &Vector3D::lerp(&self.bl, &self.br, u)
        );
    }

    pub fn get_center(&self) -> Vector3D {
        let mut ret: Vector3D = self.tl.copy();
        ret.add_vector(&self.tr);
        ret.add_vector(&self.bl);
        ret.add_vector(&self.br);
        ret.divide_scalar(4f32);
        return ret;
    }

    pub fn get_bases(&self) -> [Line; 2] {
        return [
            Line::new(&self.tl, &self.tr),
            Line::new(&self.bl, &self.br)
        ];
    }

    pub fn get_uv(&self, point: &Vector3D, u: &mut f32, v: &mut f32) -> bool {
        let mut line: Line = Line::invalid();

        let bases: [Line; 2] = self.get_bases();
        let mut resolution_sqr: f32 = 1f32;
        for idx in 0 .. 2 {
            resolution_sqr = resolution_sqr.max(bases[idx].length_sqr());
        }
        let threshold_sqr: f32 = 1f32 / resolution_sqr;

        return if self.get_u_binary_search_root(point, &mut line, u, &threshold_sqr) {
            let mut snapped: Vector3D = point.copy();
            line.snap(&mut snapped);
            *v = line.get_progress_along(&snapped);
            *v >= 0f32 && *v <= 1f32
        } else {
            false
        }
    }

    fn get_u_binary_search_root(&self, point: &Vector3D, line: &mut Line, u: &mut f32, threshold_sqr: &f32) -> bool {
        return self.get_u_binary_search(point, line, u, 0.5f32, 0.5f32, threshold_sqr);
    }

    fn get_u_binary_search(&self, point: &Vector3D, line: &mut Line, u: &mut f32, head: f32, span: f32, threshold_sqr: &f32) -> bool {
        *line = self.get_u_line(head);
        let d: f32 = line.dist_sqr(point);

        if d < 0.25f32 {
            *u = head;
            return true;
        }

        if (16f32 * span * span) <= (*threshold_sqr) {
            return false;
        }

        let half_span: f32 = span * 0.5f32;

        *line = self.get_u_line(head - half_span);
        let ld: f32 = line.dist_sqr(point);

        *line = self.get_u_line(head + half_span);
        let rd: f32 = line.dist_sqr(point);

        let ld_nan: bool = ld.is_nan();
        let rd_nan: bool = rd.is_nan();

        return if rd_nan && ld_nan {
            self.get_u_binary_search(point, line, u, head - half_span, half_span, threshold_sqr) ||
                self.get_u_binary_search(point, line, u, head + half_span, half_span, threshold_sqr)
        } else if rd_nan || ld <= rd {
            self.get_u_binary_search(point, line, u, head - half_span, half_span, threshold_sqr)
        } else {
            self.get_u_binary_search(point, line, u, head + half_span, half_span, threshold_sqr)
        }
    }

}

impl Clone for Quad {

    fn clone(&self) -> Self {
        return Self {
            tl: self.tl.copy(),
            tr: self.tr.copy(),
            bl: self.bl.copy(),
            br: self.br.copy()
        };
    }

}

impl PartialOrd<Self> for Quad {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for Quad {
    fn cmp(&self, other: &Self) -> Ordering {
        let md: f32 = self.get_center().norm_sqr();
        let od: f32 = other.get_center().norm_sqr();
        if md == od {
            Equal
        } else {
            if md < od {
                Less
            } else {
                Greater
            }
        }
    }
}
