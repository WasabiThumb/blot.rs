pub mod math;
pub mod display;
pub mod model;
mod arg;

extern crate image;
extern crate gif;
extern crate ansi_term;
extern crate termsize;
extern crate tempfile;

use std::alloc::{alloc, Layout};
use std::env;
use std::f32::consts::PI;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use ansi_term::Style;
use tempfile::{tempdir, TempDir};
use crate::arg::{Args, ArgObject};
use crate::display::camera::Camera;
use crate::display::canvas::{Canvas, ImageCanvas, PixelDraw, SampleMode};
use crate::display::canvas::{HeapCanvas, RGBA8};
use crate::display::render::{GifRenderer, GifRendererOp, GifRendererStep};
use crate::math::quad::Quad;
use crate::math::quaternion::Quaternion;
use crate::math::vector::Vector3D;
use crate::math::VectorLike;
use crate::model::Model;
use crate::model::platonic::cube;
use crate::model::sphere::uv_sphere;

fn main() {
    print_title();

    let mut arg: Args = Args::new();
    let arg_str: Vec<String> = env::args().collect();
    if arg_str.len() < 2usize {
        print_help();
        exit(0i32);
    }

    let arg_res = arg.digest(&arg_str, 1usize);
    if arg_res.is_err() {
        print_help();
        eprintln!("{}", arg_res.unwrap_err());
        eprintln!();
        exit(1i32);
    }

    let mut model: Model;
    if matches!(arg.object.unwrap(), ArgObject::Cube) {
        model = cube();
        model.transform.translate(&Vector3D::new(0f32, 0f32, 12f32));
    } else {
        model = uv_sphere(arg.lat, arg.lng);
        model.transform.translate(&Vector3D::new(0f32, 0f32, 8f32));
    }

    let output: String;
    let mut tmp_dir: Option<TempDir> = None;
    let mut tmp: bool = false;
    if arg.output.is_some() {
        output = arg.output.unwrap();
    } else {
        let res = tempdir();
        if res.is_err() {
            eprintln!("{}", res.unwrap_err());
            eprintln!();
            exit(1i32);
        }
        let td = res.unwrap();
        output = td.path().join("blot.gif").into_os_string().into_string().unwrap();
        tmp_dir = Some(td);
        tmp = true;
    }

    if arg.texture.is_some() {
        let from_file = ImageCanvas::from_file(arg.texture.unwrap());
        if from_file.is_err() {
            eprintln!("{}", from_file.unwrap_err());
            exit(1i32);
        }
        eprintln!("\x1B[s");
        render_spinning(&mut model, &from_file.unwrap(), output.clone(), arg.resolution, arg.interpolation);
    } else {
        let mut hc = HeapCanvas::new(4u32, 4u32);
        for y in 0 .. 4u32 {
            let mut color: bool = (y % 2u32) == 1u32;
            for x in 0 .. 4u32 {
                let rgb = if color {
                    RGBA8 { r: 255, g: 0, b: 255, a: 255 }
                } else {
                    RGBA8::black()
                };
                hc.set_pixel(x, y, &rgb);
                color = !color;
            }
        }
        eprintln!("\x1B[s");
        render_spinning(&mut model, &hc, output.clone(), arg.resolution, arg.interpolation);
    }

    if tmp {
        let mut f = File::open(output).unwrap();
        let buf: &mut [u8];
        unsafe {
            let ptr: *mut u8 = alloc(Layout::array::<u8>(8192usize).unwrap());
            buf = core::slice::from_raw_parts_mut(ptr, 8192usize);
        }

        let mut read: usize;
        loop {
            read = f.read(buf).unwrap();
            if read == 0usize { break; }
            std::io::stdout().write(&buf[0..read]).unwrap();
        }
        drop(tmp_dir.unwrap());
    }
}

fn print_title() {
    /*
         _     _       _
        | |__ | | ___ | |_
        | '_ \| |/ _ \| __|
        | |_) | | (_) | |_
        |_.__/|_|\___/ \__|
     */
    let submit: fn(&str, u8, u8, u8) = | line, r, g, b |
        eprintln!(
            "{}",
            ansi_term::Color::RGB(r, g, b).paint(line)
        );

    submit(" _     _       _   ", 255, 0, 255);
    submit("| |__ | | ___ | |_",  255, 0, 255);
    submit("| '_ \\| |/ _ \\| __|", 170, 0, 255);
    submit("| |_) | | (_) | |_", 85, 0, 255);
    submit("|_.__/|_|\\___/ \\__|", 0, 0, 255);
    eprintln!();
}

fn print_help() {
    let mut head = Style::new();
    head.background = Some(ansi_term::Colour::Cyan);
    head.foreground = Some(ansi_term::Colour::White);

    let mut name = Style::new();
    name.foreground = Some(ansi_term::Colour::Blue);

    let mut sep = Style::new();
    sep.foreground = Some(ansi_term::Colour::RGB(127, 0, 127));

    let mut detail = Style::new();
    detail.foreground = Some(ansi_term::Colour::RGB(255, 0, 255));

    eprintln!("{}", head.bold().paint(" USAGE "));
    eprintln!("{}", name.bold().paint("blot <object> [--args]"));
    eprintln!("{} {} {}", name.paint("object"), sep.paint("::"), detail.paint("cube or uv_sphere"));
    eprintln!("{} {} {}", name.paint("--texture"), sep.paint("::"), detail.paint("path to input texture"));
    eprintln!("{} {} {}", name.paint("--out"), sep.paint("::"), detail.paint("path to output gif file"));
    eprintln!("{} {} {}", name.paint("--resolution"), sep.paint("::"), detail.paint("gif resolution (1 - 65535)"));
    eprintln!("{} {} {}", name.paint("--interpolation"), sep.paint("::"), detail.paint("nearest, bilinear or bicubic"));
    eprintln!("{} {} {}", name.paint("--lat"), sep.paint("::"), detail.paint("latitude steps (3 - 65535) for uv_sphere"));
    eprintln!("{} {} {}", name.paint("--lng"), sep.paint("::"), detail.paint("longitude steps (3 - 65535) for uv_sphere"));
    eprintln!();
}

fn render_spinning<P: AsRef<Path>>(model: &mut Model, texture: &dyn Canvas, output: P, size: u16, int: SampleMode) {
    let mut camera: Camera = Camera::new();
    camera.set_fov(22.5f32);
    camera.set_size(size as f32, size as f32);

    let open = OpenOptions::new().write(true).create(true).open(output);
    if open.is_err() {
        eprintln!("{}", open.unwrap_err());
        exit(1i32);
    }
    let mut out_file = open.unwrap();
    let mut canvas: HeapCanvas = HeapCanvas::new(size as u32, size as u32);
    let mut renderer: GifRenderer = GifRenderer::new(&mut out_file, &mut canvas, 24u16, 48u16);

    let mut step: GifRendererStep;
    loop {
        step = renderer.step();
        match step {
            GifRendererStep::Done() => {
                break;
            }
            GifRendererStep::Error(err) => {
                panic!("{}", err);
            }
            GifRendererStep::Next(fd) => {
                eprint!("\x1B[u\x1B[1G");
                print_progress(fd.index, fd.total);

                let pc: f32 = (fd.index as f32) / (fd.total as f32);
                model.transform.rotation = Quaternion::from_euler(pc * PI * 4f32, pc * PI * 2f32, 0f32);

                struct FaceData { quad: Quad, index: usize }
                let mut faces: Vec<FaceData> = Vec::with_capacity(model.get_face_count());
                for idx in 0 .. model.get_face_count() {
                    let quad: Quad = model.get_face(idx);
                    faces.push(FaceData { quad, index: idx });
                }
                faces.sort_unstable_by(| a, b | a.quad.cmp(&b.quad));

                renderer.write(GifRendererOp::Fill(&RGBA8 { r: 0, g: 0, b: 0, a: 0 }));
                let to_render: usize = ((faces.len() as f32) / 2f32).ceil() as usize;

                for idx in (0 .. to_render).rev() {
                    let face_data: &mut FaceData = &mut faces[idx];
                    let index: usize = face_data.index;
                    let face: &mut Quad = &mut face_data.quad;

                    let mut normal: Vector3D = face.get_center();
                    normal.subtract_vector(&model.transform.translation);
                    normal.normalize();
                    let mut light_ray: Vector3D = camera.transform.translation.copy();
                    light_ray.subtract_vector(&model.transform.translation);
                    light_ray.normalize();
                    let light: f32 = light_ray.dot(&normal).powi(2i32) * 0.5f32 + 0.5f32;

                    camera.project_quad(face);

                    let mut min_x: u32 = 0;
                    let mut min_y: u32 = 0;
                    let mut max_x: u32 = 0;
                    let mut max_y: u32 = 0;
                    face.extract_int_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y);

                    let mut u: f32 = 0f32;
                    let mut v: f32 = 0f32;

                    for x in min_x ..= max_x {
                        if x >= (size as u32) { continue; }
                        for y in min_y ..= max_y {
                            if y >= (size as u32) { continue; }

                            if !face.get_uv(&Vector3D { x: x as f32, y: y as f32, z: 0f32 }, &mut u, &mut v) {
                                continue;
                            }
                            model.remap_face_uv(index, &mut u, &mut v);
                            let mut col: RGBA8 = texture.sample_uv(u, v, int);
                            if light < 1f32 {
                                col.r = ((col.r as f32) * light).floor() as u8;
                                col.g = ((col.g as f32) * light).floor() as u8;
                                col.b = ((col.b as f32) * light).floor() as u8;
                            }
                            renderer.set_pixel(x, y, &col);
                        }
                    }
                }
            }
        }
    }

    renderer.finish().unwrap();

    eprint!("\x1B[u\x1B[1G");
    print_progress(48, 48);
    eprintln!();
}

fn print_progress(cur: u16, total: u16) {
    let pc: f32 = (cur as f32) / (total as f32);

    let mut cols: u16 = termsize::get().map_or(24u16, | sz | sz.cols);
    if cols < 18u16 { cols = 18u16; }

    let mut inner: u16 = cols - 14u16;
    if inner > total { inner = total; }

    let cur_str: String = if cur >= 10u16 { format!("{}", cur) } else { format!(" {}", cur) };
    let total_str: String = if total >= 10u16 { format!("{}", total) } else { format!(" {}", total) };

    let mut cyb = Style::new().bold();
    cyb.foreground = Some(ansi_term::Colour::Cyan);
    let mut wh = Style::new();
    wh.foreground = Some(ansi_term::Colour::White);

    eprint!("{} {} {} {}", cyb.paint(cur_str), wh.paint("/"), cyb.paint(total_str), wh.paint("["));

    for i in 0 .. inner {
        let frac = (i as f32) / ((inner as f32) - 1f32);
        if total == cur || (pc >= frac && cur > 0u16) {
            eprint!("{}", cyb.paint("█"));
        } else {
            eprint!("{}", cyb.paint("░"));
        }
    }

    eprintln!(
        "{} {}{}",
        wh.paint("]"),
        cyb.paint(format!("{}", (pc * 100f32).floor() as u8)),
        wh.paint("%")
    );
}