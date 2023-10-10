extern crate image;
extern crate gif;
extern crate rand;

use std::alloc::{alloc, Layout};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use gif::Frame;
use image::{DynamicImage, ImageResult, Pixel, Rgba, RgbaImage};
use image::io::Reader as ImageReader;
use rand::prelude::random;
use crate::display::render::{GifRenderer, GifRendererOp};


pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

pub struct RGBA8F {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

impl RGBA8 {

    pub fn random(include_alpha: bool) -> RGBA8 {
        return RGBA8 {
            r: random::<u8>(),
            g: random::<u8>(),
            b: random::<u8>(),
            a: if include_alpha { random::<u8>() } else { 255u8 }
        }
    }

    pub fn black() -> RGBA8 {
        return RGBA8 { r: 0u8, g: 0u8, b: 0u8, a: 255u8 };
    }

    pub fn white() -> RGBA8 {
        return RGBA8 { r: 255u8, g: 255u8, b: 255u8, a: 255u8 };
    }

    pub fn to_float(&self) -> RGBA8F {
        return RGBA8F {
            r: self.r as f32,
            g: self.g as f32,
            b: self.b as f32,
            a: self.a as f32
        };
    }

    pub fn lerp(a: &RGBA8F, b: &RGBA8F, d: f32) -> RGBA8F {
        let k: f32 = 1f32 - d;
        return RGBA8F {
            r: ((b.r * d) + (a.r * k)),
            g: ((b.g * d) + (a.g * k)),
            b: ((b.b * d) + (a.b * k)),
            a: ((b.a * d) + (a.a * k))
        };
    }

    pub fn cubic_int(a: &RGBA8F, b: &RGBA8F, c: &RGBA8F, d: &RGBA8F, x: f32) -> RGBA8F {
        return RGBA8F {
            r: RGBA8::cubic_int_single(a.r, b.r, c.r, d.r, x),
            g: RGBA8::cubic_int_single(a.g, b.g, c.g, d.g, x),
            b: RGBA8::cubic_int_single(a.b, b.b, c.b, d.b, x),
            a: RGBA8::cubic_int_single(a.a, b.a, c.a, d.a, x)
        };
    }

    fn cubic_int_single(a: f32, b: f32, c: f32, d: f32, x: f32) -> f32 {
        return b +
            x * ( c - (2f32 * a + 3f32 * b + d) / 6f32 ) +
            x.powi(2i32) * ( ( a + c - 2f32 * b ) / 2f32 ) +
            x.powi(3i32) * ( ( b - c ) / 2f32 + (d - a) / 6f32 );
    }

}

impl RGBA8F {

    pub fn round(&self) -> RGBA8 {
        return RGBA8 {
            r: self.r.round().clamp(0f32, 255f32) as u8,
            g: self.g.round().clamp(0f32, 255f32) as u8,
            b: self.b.round().clamp(0f32, 255f32) as u8,
            a: self.a.round().clamp(0f32, 255f32) as u8
        }
    }

}

impl Display for RGBA8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum SampleMode {
    NearestNeighbor = 0,
    BiLinear = 1,
    BiCubic = 2
}

pub trait Canvas: PixelDraw {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> RGBA8;
    fn fill(&mut self, color: &RGBA8);
    fn save(&self, path: &str) -> ImageResult<()>;
    fn as_frame(&mut self, speed: i32) -> Frame;

    fn sample_uv<'a>(&self, mut u: f32, mut v: f32, mode: SampleMode) -> RGBA8 {
        let iw: u32 = self.get_width();
        let ih: u32 = self.get_height();

        if iw < 2u32 || ih < 2u32 { return RGBA8::black(); }

        let w: f32 = iw as f32;
        let h: f32 = ih as f32;

        if u >= 1f32 { u = 1f32 - f32::EPSILON; }
        let us: f32 = u * (w - 1f32);
        if v >= 1f32 { v = 1f32 - f32::EPSILON; }
        let vs: f32 = v * (h - 1f32);

        if us < 0f32 || us >= w { return RGBA8::black(); }
        if vs < 0f32 || vs >= h { return RGBA8::black(); }

        let mut usf: u32 = us.floor() as u32;
        let mut vsf: u32 = vs.floor() as u32;
        let usr: f32 = us.fract();
        let vsr: f32 = vs.fract();

        return match mode {
            SampleMode::NearestNeighbor => {
                if usr >= 0.5f32 { usf += 1u32; }
                if vsr >= 0.5f32 { vsf += 1u32; }
                self.get_pixel(usf, vsf)
            },
            SampleMode::BiLinear => {
                let tl = self.get_pixel(usf, vsf);
                let bl = self.get_pixel(usf, vsf + 1u32);
                let tr = self.get_pixel(usf + 1u32, vsf);
                let br = self.get_pixel(usf + 1u32, vsf + 1u32);

                let t: RGBA8F = RGBA8::lerp(&tl.to_float(), &tr.to_float(), usr);
                let b: RGBA8F = RGBA8::lerp(&bl.to_float(), &br.to_float(), usr);
                RGBA8::lerp(&t, &b, vsr).round()
            },
            SampleMode::BiCubic => {
                let samples: &'a mut [RGBA8F];
                unsafe {
                    let layout: Layout = Layout::array::<RGBA8F>(16).unwrap();
                    let ptr: *mut RGBA8F = alloc(layout) as *mut RGBA8F;
                    samples = core::slice::from_raw_parts_mut::<'a, RGBA8F>(ptr, 16);
                }

                for ux in -1i32 .. 3i32 {
                    let mut ui: u32 = if usf > 0u32 || ux >= 0i32 { ((usf as i32) + ux) as u32 } else { usf };
                    if ui >= iw { ui -= 1u32; }

                    for vx in -1i32 .. 3i32 {
                        let mut vi: u32 = if vsf > 0u32 || vx >= 0i32 { ((vsf as i32) + vx) as u32 } else { vsf };
                        if vi >= ih { vi -= 1u32; }

                        samples[((ux + 1i32) * 4i32 + vx + 1i32) as usize] = self.get_pixel(ui, vi).to_float();
                    }
                }

                let a: RGBA8F = RGBA8::cubic_int(&samples[0], &samples[4], &samples[8], &samples[12], usr);
                let b: RGBA8F = RGBA8::cubic_int(&samples[1], &samples[5], &samples[9], &samples[13], usr);
                let c: RGBA8F = RGBA8::cubic_int(&samples[2], &samples[6], &samples[10], &samples[14], usr);
                let d: RGBA8F = RGBA8::cubic_int(&samples[3], &samples[7], &samples[11], &samples[15], usr);
                RGBA8::cubic_int(&a, &b, &c, &d, vsr).round()
            }
        }
    }
}

pub struct HeapCanvas<'a> {
    w: u32,
    h: u32,
    data: &'a mut [u8]
}

impl HeapCanvas<'_> {

    pub fn new<'a>(w: u32, h: u32) -> HeapCanvas<'a> {
        let s: usize = (w as usize) * (h as usize) * 4usize;
        let data: &'a mut [u8];
        unsafe {
            let ptr: *mut u8 = alloc(Layout::array::<u8>(s).unwrap());
            data = core::slice::from_raw_parts_mut::<'a, u8>(ptr, s);
        }
        HeapCanvas { w, h, data }
    }

}

pub struct ImageCanvas {
    buffer: RgbaImage
}

impl ImageCanvas {
    pub fn new(img: DynamicImage) -> ImageCanvas {
        ImageCanvas {
            buffer: img.into_rgba8()
        }
    }

    pub fn from_file_assert<P>(path: P) -> ImageCanvas where P: AsRef<Path> {
        return ImageCanvas::new(ImageReader::open(path).unwrap().decode().unwrap());
    }

    pub fn from_file<P>(path: P) -> Result<ImageCanvas, ImageCanvasError> where P: AsRef<Path> {
        let open = ImageReader::open(path);
        if open.is_err() {
            let msg = format!("{}", open.err().unwrap()).into_boxed_str();
            return Err(ImageCanvasError { msg });
        }
        let decode = open.unwrap().decode();
        if decode.is_err() {
            let msg = format!("{}", decode.err().unwrap()).into_boxed_str();
            return Err(ImageCanvasError { msg });
        }
        return Ok(ImageCanvas::new(decode.unwrap()));
    }
}

pub struct ImageCanvasError {
    msg: Box<str>
}

impl Debug for ImageCanvasError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Display for ImageCanvasError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for ImageCanvasError { }

impl PixelDraw for ImageCanvas {
    fn set_pixel(&mut self, x: u32, y: u32, color: &RGBA8) {
        self.buffer.put_pixel(x, y, Rgba::<u8>::from([ color.r, color.g, color.b, color.a ]));
    }
}

impl Canvas for ImageCanvas {
    fn get_width(&self) -> u32 {
        self.buffer.width()
    }

    fn get_height(&self) -> u32 {
        self.buffer.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> RGBA8 {
        let dat: &[u8] = self.buffer.get_pixel(x, y).channels();
        return RGBA8 {
            r: dat[0],
            g: dat[1],
            b: dat[2],
            a: if dat.len() > 3usize {
                dat[3]
            } else {
                255
            }
        }
    }

    fn fill(&mut self, color: &RGBA8) {
        for y in 0 .. self.get_height() {
            for x in 0 .. self.get_width() {
                self.set_pixel(x, y, color);
            }
        }
    }

    fn save(&self, path: &str) -> ImageResult<()> {
        self.buffer.save(path)
    }

    fn as_frame(&mut self, speed: i32) -> Frame {
        let w: u32 = self.buffer.width();
        if w > u16::MAX as u32 { panic!("Width ({}) eclipses 16-bit integer limit", w) }
        let h: u32 = self.buffer.height();
        if h > u16::MAX as u32 { panic!("Height ({}) eclipses 16-bit integer limit", h) }
        Frame::from_rgba_speed(w as u16, h as u16, self.buffer.to_vec().as_mut_slice(), speed)
    }
}

impl Debug for ImageCanvas {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageCanvas({}, {})", self.buffer.width(), self.buffer.height())
    }
}

impl HeapCanvas<'_> {

    fn get_size(&self) -> usize {
        self.data.len()
    }

    fn compute_index(&self, x: u32, y: u32) -> usize {
        ((y as usize) * (self.w as usize) * 4usize) + ((x as usize) * 4usize)
    }

    fn compute_safe_index(&self, x: u32, y: u32) -> usize {
        let ret: usize = self.compute_index(x, y);
        let max: usize = self.get_size();
        if ret >= max {
            panic!("HeapCanvas indexed at {} for length {}", ret, max)
        }
        ret
    }

}

impl PixelDraw for HeapCanvas<'_> {
    fn set_pixel(&mut self, x: u32, y: u32, color: &RGBA8) {
        let idx: usize = self.compute_safe_index(x, y);
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = color.a;
    }
}

impl Canvas for HeapCanvas<'_> {
    fn get_width(&self) -> u32 {
        self.w
    }

    fn get_height(&self) -> u32 {
        self.h
    }

    fn get_pixel(&self, x: u32, y: u32) -> RGBA8 {
        let idx: usize = self.compute_safe_index(x, y);
        return RGBA8 {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
            a: self.data[idx + 3]
        }
    }

    fn fill(&mut self, color: &RGBA8) {
        let components: [ u8; 4 ] = [ color.r, color.g, color.b, color.a ];
        let n: usize = (self.w as usize) * (self.h as usize);
        for q in 0 .. n {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &components as *const u8,
                    &mut self.data[q << 2] as *mut u8,
                    4usize
                );
            }
        }
    }

    fn save(&self, path: &str) -> ImageResult<()> {
        let img = RgbaImage::from_raw(self.w, self.h, self.data.to_vec()).unwrap();
        img.save(path)
    }

    fn as_frame(&mut self, speed: i32) -> Frame {
        if self.w > u16::MAX as u32 { panic!("Width ({}) eclipses 16-bit integer limit", self.w) }
        if self.h > u16::MAX as u32 { panic!("Height ({}) eclipses 16-bit integer limit", self.h) }
        Frame::from_rgba_speed(self.w as u16, self.h as u16, self.data, speed)
    }
}

impl Drop for HeapCanvas<'_> {

    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.data);
        }
    }

}

pub trait PixelDraw {
    fn set_pixel(&mut self, x: u32, y: u32, col: &RGBA8);
}

impl PixelDraw for GifRenderer<'_> {

    fn set_pixel(&mut self, x: u32, y: u32, col: &RGBA8) {
        self.write(GifRendererOp::SetPixel(x, y, col));
    }

}
