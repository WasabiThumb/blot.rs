use crate::math::vector::Vector3D;
use crate::math::VectorLike;

enum LineMode {
    Invalid,
    Horizontal(f32), // y
    Vertical(f32), // x
    Diagonal(f32, f32, bool) // m, b, bound_by_x
}

pub struct Line {
    min: f32,
    max: f32,
    reverse: bool, // true: 1 at min, 0 at max. false: 0 at min, 1 at max.
    mode: LineMode
}

impl Line {

    pub fn invalid() -> Line {
        return Line { min: 0f32, max: 0f32, reverse: false, mode: LineMode::Invalid };
    }

    pub fn new(a: &Vector3D, b: &Vector3D) -> Line {
        let dy: f32 = b.y - a.y;
        if dy.abs() <= f32::EPSILON {
            if (b.x - a.x).abs() <= f32::EPSILON {
                return Line::invalid();
            }
            let xm: f32 = a.x.max(b.x);
            return Line {
                min: a.x.min(b.x),
                max: xm,
                reverse: a.x == xm,
                mode: LineMode::Horizontal(b.y)
            };
        }

        let dx: f32 = b.x - a.x;
        if dx.abs() <= f32::EPSILON {
            let ym: f32 = a.y.max(b.y);
            return Line {
                min: a.y.min(b.y),
                max: ym,
                reverse: a.y == ym,
                mode: LineMode::Vertical(b.x)
            };
        }

        let lm: f32 = dy / dx;
        let lb: f32 = b.y - (lm * b.x);

        let ret: Line;

        if dx >= dy {
            let xm: f32 = a.x.max(b.x);
            ret = Line {
                min: a.x.min(b.x),
                max: xm,
                reverse: a.x == xm,
                mode: LineMode::Diagonal(lm, lb, true)
            };
        } else {
            let ym: f32 = a.y.max(b.y);
            ret = Line {
                min: a.y.min(b.y),
                max: ym,
                reverse: a.y == ym,
                mode: LineMode::Diagonal(lm, lb, false)
            };
        }
        ret
    }

    pub fn bound_of(&self, vec: &Vector3D) -> f32 {
        return match self.mode {
            LineMode::Invalid => f32::NAN,
            LineMode::Horizontal(_) => vec.x,
            LineMode::Vertical(_) => vec.y,
            LineMode::Diagonal(_, _, bound_by_x) => {
                if bound_by_x {
                    vec.x
                } else {
                    vec.y
                }
            }
        };
    }

    pub fn in_bounds(&self, vec: &Vector3D) -> bool {
        let bound: f32 = self.bound_of(vec);
        return bound >= self.min && bound <= self.max;
    }

    pub fn snap(&self, out: &mut Vector3D) -> bool {
        return match self.mode {
            LineMode::Invalid => false,
            LineMode::Horizontal(y) => {
                // let ox: f32 = out.x;
                // if ox < self.min || ox > self.max { return false; }
                out.y = y;
                true
            },
            LineMode::Vertical(x) => {
                // let oy: f32 = out.y;
                // if oy < self.min || oy > self.max { return false; }
                out.x = x;
                true
            },
            LineMode::Diagonal(m, b, _) => {
                let nm: f32 = -1f32 / m;
                let nb: f32 = out.y - (nm * out.x);

                let nx: f32 = (nb - b) / (m - nm);
                out.x = nx;
                out.y = (m * nx) + b;
                // self.in_bounds(out)
                true
            }
        }
    }

    pub fn get_progress_along(&self, point: &Vector3D) -> f32 {
        let mut ret: f32 = (self.bound_of(point) - self.min) / (self.max - self.min);
        if self.reverse {
            ret = 1f32 - ret;
        }
        return ret;
    }

    pub fn dist_sqr(&self, point: &Vector3D) -> f32 {
        let mut snapped: Vector3D = point.copy();
        return if !self.snap(&mut snapped) {
            f32::NAN
        } else {
            snapped.subtract_vector(point);
            snapped.norm_sqr()
        }
    }

    pub fn length_sqr(&self) -> f32 {
        return match self.mode {
            LineMode::Invalid => {
                0f32
            },
            LineMode::Horizontal(_) | LineMode::Vertical(_) => {
                (self.max - self.min).powi(2i32)
            },
            LineMode::Diagonal(m, b, bound_by_x) => {
                let x1: f32;
                let y1: f32;
                let x2: f32;
                let y2: f32;

                if bound_by_x {
                    x1 = self.min;
                    x2 = self.max;
                    y1 = (m * x1) + b;
                    y2 = (m * x2) + b;
                } else {
                    y1 = self.min;
                    y2 = self.max;
                    x1 = (self.min - b) / m;
                    x2 = (self.max - b) / m;
                }

                (x2 - x1).powi(2i32) + (y2 - y1).powi(2i32)
            }
        }
    }

}
