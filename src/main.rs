use std::env;
use std::num::ParseIntError;
use std::thread;
use std::time::Duration;

use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEc, EcResult};


// Milliseconds per second
const UNIT_MS: u16 = 1000;
const N_LEDS: u8 = 8;
// ticks per second
const TICKRATE: u16 = 50;
// how often to refresh in case computer enters sleep mode or something
const REFRESH_PERIOD: u16 = 10000;
const BLINK_PERIOD: u16 = (UNIT_MS / TICKRATE) / 2;
const SPIN_PERIOD: u16 = (UNIT_MS / TICKRATE) * 5;

const OFF: RgbS = RgbS{ r: 0, g: 0, b: 0 };

enum Animation {
    Solid { color: RgbS },
    Blink {
        colors: Vec<RgbS>,
        period: u16,    // unit in ticks
        current_color_index: u8,
        on: bool,
    },
    SmoothSpin {
        colors: Vec<RgbS>,
        period: u16,              // unit in ticks
        current_rotation: f32,    // unit in leds
    },
}


impl Animation {
    fn from_cli(modestr: &str, colors: Vec<RgbS>) -> Self {
        if colors.len() > N_LEDS as usize {
            panic!("There can't be more colors than LEDS!")
        }
        
        match modestr {
            "solid" => {
                let color = colors
                    .first()
                    .copied()
                    .expect("Solid mode requires color");
                Animation::Solid{ color }
            },
            "blink" => {
                Animation::Blink{
                    colors,
                    period: BLINK_PERIOD * TICKRATE,
                    current_color_index: 0,
                    on: false,
                }
            },
            "smoothspin" => Animation::SmoothSpin {
                colors,
                period: SPIN_PERIOD,
                current_rotation: 0.0,
            },
            _ => panic!("Unknown animation mode."),
        }
    }

    // stepper function
    fn step(&mut self, leds: &mut [RgbS]) {
        match self {
            Animation::Solid { color } => {
                thread::sleep(Duration::from_millis(REFRESH_PERIOD.into()));
                
                for led in leds {
                    *led = color.clone();
                }
            },
            Animation::Blink {
                colors,
                period,
                current_color_index,
                on,
            } => {
                thread::sleep(Duration::from_millis(*period as u64));

                if *on {
                    for led in leds {
                        *led = OFF;
                    }
                    
                    if (*current_color_index as usize) >= (*colors).len() - 1 {
                        *current_color_index = 0;
                    } else {
                        *current_color_index += 1;
                    }
                    
                } else {
                    let current_color: RgbS = match colors.get_mut(*current_color_index as usize) {
                        Some(color) => *color,
                        None => panic!("Index {} is out of bounds", *current_color_index),
                    };
                    
                    for led in leds {                        
                        *led = current_color.clone();
                    }
                }

                *on = !*on;
                
            },
            Animation::SmoothSpin {
                colors,
                period,
                current_rotation,  
            } => {
                thread::sleep(Duration::from_millis(TICKRATE.into()));

                let step = if *period == 0 {
                    0.0
                } else {
                    N_LEDS as f32 / *period as f32
                };

                *current_rotation = (*current_rotation + step) % N_LEDS as f32;

                for (i, led) in leds.iter_mut().enumerate() {
                    let sample_pos = (*current_rotation + i as f32) % N_LEDS as f32;
                    *led = sample_gradient(colors, sample_pos);
                }
            },
        }
    }
}


#[derive(Debug)]
enum HexParseError {
    WrongLength,
    InvalidDigit,
}

impl From<ParseIntError> for HexParseError {
    fn from(_: ParseIntError) -> Self {
        HexParseError::InvalidDigit
    }
}

// linear interpolation between two colors
fn lerp(a: RgbS, b: RgbS, t: f32) -> RgbS {
    RgbS {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
    }
}

// samples the true color gradient wheel at an led
fn sample_gradient(colors: &[RgbS], pos: f32) -> RgbS {
    let n = colors.len();

    let scaled = pos * n as f32 / N_LEDS as f32;
    let idx = scaled.floor() as usize % n;
    let next_idx = (idx + 1) % n;
    let t = scaled - scaled.floor();

    lerp(colors[idx], colors[next_idx], t)
}

fn parse_hex(s: &str) -> Result<RgbS, HexParseError> {
    if s.len() != 6 {
        return Err(HexParseError::WrongLength);
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;

    Ok(RgbS { r, g, b })
}

fn args_to_rgbs(args: Vec<String>) -> Result<Vec<RgbS>, HexParseError> {
    args.into_iter()
        .skip(1)
        .map(|s| parse_hex(&s))
        .collect()
}

fn main() -> EcResult<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let animation_modestr = args.get(0)
        .expect("Expected an animation argument, and several color arguments.")
        .as_str()
        .to_owned();

    let colors: Vec<RgbS> = args_to_rgbs(args).unwrap_or_else(|e| panic!("Failed to parse color argument: {e:?}"));

    let mut animation = Animation::from_cli(&animation_modestr, colors);

    let ec = CrosEc::new();
    
    let mut leds: [RgbS; N_LEDS as usize] = [OFF; N_LEDS as usize];
        
    loop {
        animation.step(&mut leds);

        if let Err(e) = ec.rgbkbd_set_color(0, leds.to_vec()) {
            eprintln!("Error setting lights: {:?}", e);
        }
    }
        
}
