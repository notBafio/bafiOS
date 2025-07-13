#![no_std]

use libk;

extern crate alloc;

pub mod draw;
pub mod psf;
pub mod targa;
pub mod widgets;

pub fn kui_ceil(x: f32) -> f32 {
    let int_part = x as i32;
    if x > 0.0 && x > int_part as f32 {
        (int_part + 1) as f32
    } else {
        int_part as f32
    }
}
