#![feature(io, type_ascription, conservative_impl_trait)]
#![allow(unused_imports, unused_macros, dead_code)]

extern crate num;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use num::complex::*;
use std::ops::Sub;

macro_rules! const_assert {
    ($($condition:expr),+ $(,)*) => {{
        let _ = [(); 0 - !($($condition)&&+) as usize];
    }};
    ($label:ident; $($rest:tt)+) => {{
        #[allow(dead_code, non_snake_case)]
        fn $label() {
            const_assert!($($rest)+);
        }
    }};
}

macro_rules! assert_eq_size {
    ($x:ty, $($xs:ty),+ $(,)*) => {{
        $(let _ = transmute::<$x, $xs>;)+
    }};
    ($lavel:ident; $($rest:tt)+) => {{
        #[allow(dead_code, non_snake_case)]
        fn $label() {
            assert_eq_size!($($rest)+);
        }
    }};
}

pub const CONSOLE_WIDTH  : usize = 80;
pub const CONSOLE_HEIGHT : usize = 24;

pub struct Config {
    filename : String, 
}

impl Config {
    pub fn new(mut args : std::env::Args) -> Result<Config, impl Into<String>> { 
        let filename : String;
        let progname = args.nth(0).unwrap_or("afluid-rs".into());
        let mut iterator = args.skip(1).peekable();
        
        while let Some(arg) = iterator.next() {
            match &arg[..] {
                "-h" | "--help" => { usage(progname); return Err("") },
                "-v" | "--version" => { println!("{}", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")); return Err("") } 
                "-d" | "--debug"   => { println!("TODO"); return Err("") }
                _ => { filename = arg.into(); return Ok(Config { filename }) }, // todo
            }
        };

        usage(progname);
        Err("Didn't get a filename")
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Particle {
    pub x_pos   : i64,
    pub y_pos   : i64,
    pub x_fce   : f64,
    pub y_fce   : f64,
    pub x_vel   : f64,
    pub y_vel   : f64,
    pub density : f64,
    pub wall    : i32,
    // pub c_pos   : Complex<i64>, 
    // pub c_fce   : Complex<f64>,
    // pub c_vel   : Complex<f64>,
}

#[derive(Debug, Default)]
pub struct Particles {
    pub particles : Vec<RefCell<Particle>>, 
    pub total : usize,
}

impl Particles {
    #[inline]
    fn new() -> Particles {
        Particles {
            // particles : Rc::new(RefCell::new(Vec::<Particle>::with_capacity(CONSOLE_HEIGHT * CONSOLE_WIDTH * 2))),
            particles : Vec::<RefCell<Particle>>::with_capacity(CONSOLE_HEIGHT * CONSOLE_WIDTH * 2),
            total : 0,
        }
    }
}

// fn usage(mut args : std::env::Args) {
fn usage(name : String) {
    // let name : String = args.nth(0).unwrap_or("afluid-rs".into());
    {
        println!("{:?} Ascii fluid simulation [{}] [UNRELEASED]",
                 name, option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
        println!("\t -h, {:<14} show this message",         "--help");
        println!("\t -v, {:<14} project version",           "--help");
        println!("\t -l, {:<14} provide the file",          "--load");
        println!("\t -d, {:<14} force debug output {:>14}", "--debug", "[NONE]");
        println!("\n Example: \n\t {} -l [path]\n", name);
    }
}

fn join_mut<'a, T>(f : &'a mut [T], s : &'a mut [T]) -> Option<&'a mut [T]> {
    let fl = f.len();
    if f[fl..].as_mut_ptr() == s.as_mut_ptr() {
        unsafe {
            Some(::std::slice::from_raw_parts_mut(f.as_mut_ptr(), fl + s.len()))
        }
    } else { None }
}

pub fn prepare_data(config : Config) -> Result<Particles, Box<Error>> {
    let mut data : Particles = Particles::new();

    for _ in 0..(CONSOLE_HEIGHT * CONSOLE_WIDTH * 2) {
        data.particles.push(Default::default());
    }

    let mut p_c : usize = 0;

    let mut y_idx = 0;
    let mut x_idx = 0;

    // let mut ic_pos : Complex<i64> = Default::default();

    for c in File::open(config.filename)?.chars() {
        match c {
            Ok('\n') => { 
                x_idx = 0;
                y_idx += 2;
                // ic_pos = Complex { re: 0, im: ic_pos.im + 2 };
            } 
            Ok(' ') => {  },
            Ok('#') => { 
                data.particles[p_c + 0].borrow_mut().wall = 1;
                data.particles[p_c + 1].borrow_mut().wall = 1;
            },
            _ => {
                data.particles[p_c].borrow_mut().x_pos = x_idx;
                data.particles[p_c].borrow_mut().y_pos = y_idx;
                // data.particles[p_c].borrow_mut().c_pos = ic_pos;

                data.particles[p_c + 1].borrow_mut().x_pos = x_idx;
                data.particles[p_c + 1].borrow_mut().y_pos = y_idx + 1;
                // data.particles[p_c + 1].borrow_mut().c_pos = Complex::new(ic_pos.re, ic_pos.im + 1); 
                
                p_c += 2;
                data.total = p_c;
            },
        };
        x_idx += 1;
    }

    Ok(data)
}
