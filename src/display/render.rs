extern crate image;
extern crate gif;

use std::fs::File;
use gif::{DisposalMethod, Encoder, EncodingError, Repeat};
use crate::display::canvas::{Canvas, RGBA8};

pub struct GifRenderer<'a> {
    encoder: Encoder<&'a mut File>,
    canvas: &'a mut dyn Canvas,
    frame_count: u16,
    head: u16,
    delay: GifRendererDelayData
}

struct GifRendererDelayData {
    delay_floor: u16,
    delay_rem: f32,
    delay_last: u16,
    head: f32,
    perfect: bool
}

impl GifRendererDelayData {

    fn new(rate: u16) -> Self {
        let delay: f32 = (100u16 as f32) / (rate as f32);
        let delay_floor: f32 = delay.floor();
        let delay_floor_int: u16 = delay_floor as u16;
        let delay_rem: f32 = delay_floor - delay;
        return Self {
            delay_floor: delay_floor_int,
            delay_rem,
            delay_last: delay_floor_int,
            head: 0f32,
            perfect: delay_rem == 0f32
        }
    }

    fn next_delay0(&mut self) -> u16 {
        if self.perfect {
            return self.delay_floor;
        }
        self.head += self.delay_rem;
        return if self.head >= 0.5f32 {
            self.head -= 1f32;
            self.delay_floor + 1u16
        } else {
            self.delay_floor
        }
    }

    pub fn next_delay(&mut self) -> u16 {
        let d: u16 = self.next_delay0();
        self.delay_last = d;
        return d;
    }

    pub fn last_delay(&self) -> u16 {
        return self.delay_last;
    }

}

pub enum GifRendererStep {
    Done(),
    Error(String),
    Next(GifFrameData)
}

pub struct GifFrameData {
    pub width: u32,
    pub height: u32,
    pub index: u16,
    pub total: u16,
    pub delta: f32
}

pub enum GifRendererOp<'t> {
    Fill(&'t RGBA8),
    SetPixel(u32, u32, &'t RGBA8)
}

impl<'a> GifRenderer<'a> {
    pub fn new(file: &'a mut File, canvas: &'a mut dyn Canvas, frame_rate: u16, frame_count: u16) -> Self {
        let mut encoder = Encoder::new(file, canvas.get_width() as u16, canvas.get_height() as u16, &[]).unwrap();
        encoder.set_repeat(Repeat::Infinite).unwrap();
        Self { encoder, canvas, frame_count, head: 0u16, delay: GifRendererDelayData::new(frame_rate) }
    }

    fn flush(&mut self) -> Result<(), EncodingError> {
        if self.head > 0 {
            let mut f = self.canvas.as_frame(10);
            f.delay = self.delay.next_delay();
            f.dispose = DisposalMethod::Background;
            self.encoder.write_frame(&f)
        } else {
            Ok(())
        }
    }

    pub fn step(&mut self) -> GifRendererStep {
        let head: u16 = self.head;
        if head >= self.frame_count {
            return GifRendererStep::Done()
        }
        let res = self.flush();
        self.head = head + 1;
        if res.is_err() {
            GifRendererStep::Error(format!("{}", res.unwrap_err()))
        } else {
            GifRendererStep::Next(GifFrameData {
                width: self.canvas.get_width(),
                height: self.canvas.get_height(),
                index: head,
                total: self.frame_count,
                delta: (self.delay.last_delay() as f32) / 100f32,
            })
        }
    }

    pub fn write(&mut self, op: GifRendererOp) {
        if self.head < 1 || self.head > self.frame_count {
            return;
        }
        match op {
            GifRendererOp::Fill(col) => {
                self.canvas.fill(col);
            }
            GifRendererOp::SetPixel(x, y, col) => {
                self.canvas.set_pixel(x, y, col);
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), EncodingError> {
        let res = self.flush();
        if res.is_err() {
            return res.map(|_q| unreachable!());
        }
        self.head = self.frame_count;
        Ok(())
    }
}
