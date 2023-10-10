use std::fmt::{Debug, Display, Formatter};
use ansi_term::Style;
use std::error::Error;
use crate::display::canvas::SampleMode;

pub enum ArgObject {
    Cube,
    UvSphere
}

enum ArgSelector {
    Unset,
    Texture,
    Output,
    Resolution,
    Interpolation,
    LatitudeSteps,
    LongitudeSteps
}

pub struct Args {
    pub object: Option<ArgObject>,
    pub texture: Option<String>,
    pub output: Option<String>,
    pub resolution: u16,
    pub interpolation: SampleMode,
    pub lat: u16,
    pub lng: u16,

    active_selector: ArgSelector
}

impl Args {
    pub fn new() -> Self {
        Self {
            object: None,
            texture: None,
            output: None,
            resolution: 256u16,
            interpolation: SampleMode::BiLinear,
            lat: 128u16,
            lng: 64u16,
            active_selector: ArgSelector::Unset
        }
    }

    pub fn digest(&mut self, args: &Vec<String>, index: usize) -> Result<(), ArgError> {
        if index >= args.len() {
            if self.object.is_none() {
                return Err(ArgError { name: String::from("object"), issue: String::from("Argument is required") });
            }
            return Ok(());
        }
        if index == 1 {
            let ob_str = &args[index];

            if ob_str.eq_ignore_ascii_case("cube") {
                self.object = Some(ArgObject::Cube);
            } else if ob_str.eq_ignore_ascii_case("uv_sphere") || ob_str.eq_ignore_ascii_case("uv") {
                self.object = Some(ArgObject::UvSphere);
            } else {
                return Err(ArgError { name: String::from("object"), issue: String::from("Not one of cube, uv_sphere") });
            }
        } else {
            if matches!(self.active_selector, ArgSelector::Unset) {
                let res = self.digest_selector(&args[index]);
                if res.is_err() { return res; }
            } else {
                let res = self.digest_value(&args[index]);
                if res.is_err() { return res; }
            }
        }
        return self.digest(args, index + 1usize);
    }

    fn digest_selector(&mut self, selector: &String) -> Result<(), ArgError> {
        if selector.eq_ignore_ascii_case("--texture") || selector.eq_ignore_ascii_case("-t") {
            self.active_selector = ArgSelector::Texture;
        } else if selector.eq_ignore_ascii_case("--out") || selector.eq_ignore_ascii_case("-o") {
            self.active_selector = ArgSelector::Output;
        } else if selector.eq_ignore_ascii_case("--resolution") || selector.eq_ignore_ascii_case("-r") {
            self.active_selector = ArgSelector::Resolution;
        } else if selector.eq_ignore_ascii_case("--interpolation") || selector.eq_ignore_ascii_case("-i") {
            self.active_selector = ArgSelector::Interpolation;
        } else if selector.eq_ignore_ascii_case("--lat") {
            self.active_selector = ArgSelector::LatitudeSteps;
        } else if selector.eq_ignore_ascii_case("--lng") {
            self.active_selector = ArgSelector::LongitudeSteps;
        } else {
            return Err(ArgError { name: String::from("selector"), issue: format!("Unrecognized selector {}", selector) });
        }
        Ok(())
    }

    fn digest_value(&mut self, value: &String) -> Result<(), ArgError> {
        match self.active_selector {
            ArgSelector::Texture => {
                self.texture = Some(value.clone());
            },
            ArgSelector::Output => {
                self.output = Some(value.clone());
            },
            ArgSelector::Resolution => {
                let parse = value.parse::<u16>();
                if parse.is_err() {
                    return Err(ArgError { name: String::from("resolution"), issue: format!("Invalid resolution ({})", value) });
                }
                self.resolution = parse.unwrap();
            },
            ArgSelector::Interpolation => {
                if value.eq_ignore_ascii_case("nearest") {
                    self.interpolation = SampleMode::NearestNeighbor;
                } else if value.eq_ignore_ascii_case("bilinear") {
                    self.interpolation = SampleMode::BiLinear;
                } else if value.eq_ignore_ascii_case("bicubic") {
                    self.interpolation = SampleMode::BiCubic;
                } else {
                    return Err(ArgError { name: String::from("interpolation"), issue: format!("Unrecognized sample mode ({})", value) });
                }
            },
            ArgSelector::LatitudeSteps => {
                let parse = value.parse::<u16>();
                if parse.is_err() {
                    return Err(ArgError { name: String::from("lat"), issue: format!("Invalid latitude step count ({})", value) });
                }
                self.lat = parse.unwrap();
            },
            ArgSelector::LongitudeSteps => {
                let parse = value.parse::<u16>();
                if parse.is_err() {
                    return Err(ArgError { name: String::from("lng"), issue: format!("Invalid longitude step count ({})", value) });
                }
                self.lng = parse.unwrap();
            },
            ArgSelector::Unset => panic!("Selector unset while digesting value!")
        }
        self.active_selector = ArgSelector::Unset;
        Ok(())
    }
}

pub struct ArgError {
    name: String,
    issue: String
}

impl Debug for ArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.issue)
    }
}

impl Display for ArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut block = Style::new().bold();
        block.foreground = Some(ansi_term::Colour::White);
        block.background = Some(ansi_term::Colour::Red);

        let mut detail = Style::new();
        detail.foreground = Some(ansi_term::Colour::Red);
        detail.is_dimmed = true;

        write!(
            f,
            "{}{}{}",
            block.paint(" ERROR "),
            detail.paint(format!(" ({}) ", self.name)),
            ansi_term::Colour::Red.paint(self.issue.clone())
        )
    }
}

impl Error for ArgError { }
