#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use uefi::{
    Result,
    prelude::*,
    proto::console::gop::{BltOp, BltPixel, BltRegion::Full, GraphicsOutput},
};

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    fn init_flat(width: usize, height: usize, color: BltPixel) -> Self {
        Self {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    fn blt(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    draw_purple();

    Status::SUCCESS
}

fn draw_purple() -> Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let buffer = Buffer::init_flat(width, height, BltPixel::new(0, 0, 255));

    buffer.blt(&mut gop)?;

    boot::stall(Duration::from_secs(10));

    Ok(())
}
