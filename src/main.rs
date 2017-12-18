#[allow(dead_code)]

extern crate afluid;
extern crate core;
extern crate num;

use afluid::*;

use std::io::{self, Write};
use std::thread;
use std::env;
use std::time::Duration;

use std::ptr;
use std::mem;

use num::complex::*;

const GRAVITY   : f64 = 1.0;
const PRESSURE  : f64 = 3.0;
const VISCOSITY : f64 = 6.0;

const SCREEN : usize = CONSOLE_WIDTH * CONSOLE_HEIGHT * 2 + 1;

static mut P_DIST : f64 = 0.0;
static mut P_INT  : f64 = 0.0;
    
static mut C_PAR_DIST : Complex<i64> = Complex { re : 0, im : 0 }; 

static mut X : i64 = 0;
static mut Y : i64 = 0;


unsafe fn run(data_un : &mut Particles) {
    let mut scr_buf : [u8;       SCREEN] = mem::uninitialized();
    let mut arr     : [Particle; SCREEN] = mem::uninitialized();

    for i in 0..SCREEN-1 {
        ptr::write(&mut arr[i], *data_un.particles[i].borrow());
    }
    
    loop {
        let mut limit : usize = 0;
        
        for particle1 in data_un.particles.iter_mut() {

            let mut particle1 = particle1.borrow_mut(); 

            particle1.density = particle1.wall as f64 * 9f64;

            particle1.x_fce = 0f64;
            particle1.y_fce = GRAVITY;

            // particle1.c_fce = Complex {
            //     re : 0f64, im : GRAVITY
            // };

            for particle2 in arr.iter() {
                C_PAR_DIST = Complex {
                    // re : particle1.c_pos.re - particle2.c_pos.re,
                    // im : particle1.c_pos.im - particle2.c_pos.im,
                    re : particle1.x_pos - particle2.x_pos,
                    im : particle1.y_pos - particle2.y_pos, 
                };

                P_DIST = ((C_PAR_DIST.re as f64).powf(2f64) + (C_PAR_DIST.im as f64).powf(2f64)).sqrt(); 
                P_INT = P_DIST / 2f64 - 1f64;

                if (1f64 - P_INT).floor() > 0f64 {
                    particle1.density += P_INT.powf(2f64);

                    particle1.x_fce += P_INT * 
                        ( C_PAR_DIST.re as f64 * 
                            (3f64 - particle1.density - particle2.density) * 
                            PRESSURE + particle1.x_vel * VISCOSITY - particle2.x_vel as f64 * VISCOSITY
                        ) / particle1.density;
                    
                    particle1.y_fce += P_INT * 
                        ( C_PAR_DIST.im as f64 * 
                            (3f64 - particle1.density - particle2.density) * 
                            PRESSURE + particle1.y_vel * VISCOSITY - particle2.y_vel as f64 * VISCOSITY
                        ) / particle1.density;

                    // particle1.c_fce = Complex {
                    //     re : particle1.c_fce.re + P_INT * ( C_PAR_DIST.re as f64 * 
                    //         (3f64 - particle1.density - particle2.density) * 
                    //         PRESSURE + particle1.c_vel.re * VISCOSITY - 
                    //         particle2.c_vel.re as f64 * VISCOSITY) / particle1.density,
                    //     im : particle1.c_fce.im + P_INT * ( C_PAR_DIST.re as f64 * 
                    //         (3f64 - particle1.density - particle2.density) * 
                    //         PRESSURE + particle1.c_vel.im * VISCOSITY - 
                    //         particle2.c_vel.im as f64 * VISCOSITY) / particle1.density,
                    // }
                }

                if limit == 6841 { break; }
                limit += 1;
            }

            if particle1.wall > 0 {
                // if (particle1.c_fce.re.powf(2f64) + particle1.c_fce.im.powf(2f64)).sqrt() < 4.2 {
                if (particle1.x_fce.powf(2f64) + particle1.y_fce.powf(2f64)).sqrt() < 4.2 {
                    particle1.x_vel += particle1.x_fce / 10f64;
                    particle1.y_vel += particle1.y_fce / 10f64;
                    // particle1.c_vel = Complex {
                    //     re : particle1.c_pos.re as f64 + particle1.c_fce.re / 10f64, // particle1.c_fce.re / 10f64,
                    //     im : particle1.c_pos.im as f64 + particle1.c_fce.im / 10f64, // particle1.c_fce.im / 11f64,
                    // }
                } else {
                    particle1.x_vel += particle1.x_fce / 11f64;
                    particle1.y_vel += particle1.y_fce / 11f64;
                    // particle1.c_vel = Complex {
                    //     re : particle1.c_vel.re + particle1.c_fce.re / 11f64, // particle1.c_fce.re / 11f64,
                    //     im : particle1.c_vel.im + particle1.c_fce.im / 11f64, // particle1.c_fce.im / 11f64,
                    // }
                }

                particle1.x_pos += particle1.x_vel.round() as i64;
                particle1.y_pos += particle1.y_vel.round() as i64;
                // particle1.c_pos += Complex {
                //     re : particle1.c_vel.re.round() as i64,
                //     im : particle1.c_vel.im.round() as i64,
                // };
            }

            X = particle1.x_pos;
            Y = particle1.y_pos;
            // x = particle1.c_pos.re;
            // y = particle1.c_pos.im;

            let mut sb_i = X + CONSOLE_WIDTH as i64 * Y; 

            if Y >= 0 && Y < CONSOLE_HEIGHT as i64 - 1 && X >= 0 && X < CONSOLE_WIDTH as i64 - 1 {
                scr_buf[sb_i as usize                    ] |= 8;
                scr_buf[sb_i as usize + 1                ] |= 4;
                scr_buf[sb_i as usize + CONSOLE_WIDTH    ] |= 2;
                scr_buf[sb_i as usize + CONSOLE_WIDTH + 1] |= 1;
            }
        }

        let charset = b" '`-.|//,\\|\\_\\/#"[..].to_owned();
        
        for sb_i in 0..(CONSOLE_HEIGHT * CONSOLE_WIDTH) {
            scr_buf[sb_i as usize] = 
                if sb_i % CONSOLE_WIDTH != CONSOLE_WIDTH - 1 { 
                    charset[(scr_buf[sb_i as usize] % 16) as usize] 
                } else { 
                    '\n' as u8 
                }
        }

        io::stdout().write(b"\x1b[1;1H").unwrap();
        io::stdout().write(&scr_buf).unwrap();

        thread::sleep(Duration::from_millis(70));
    }
}


fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("error: {}", err.into());
        std::process::exit(1);
    });
    
    io::stdout().write(b"\x1b[2J").unwrap();

    unsafe {
        run(&mut prepare_data(config).unwrap_or_else(|err| {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }));
    }
}
