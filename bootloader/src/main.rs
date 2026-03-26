#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use common::BootInfo;
use core::time::Duration;
use uefi::{
    Result,
    boot::MemoryType,
    mem::memory_map::MemoryMap,
    prelude::*,
    println,
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

    println!("Bootloader initialized");
    println!("Getting Framebuffer");

    // Framebuffer
    let gop_handle =
        boot::get_handle_for_protocol::<GraphicsOutput>().expect("Failed to get gop-handle");
    let mut gop =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).expect("Failed to open gop");
    let gop_info = gop.current_mode_info();

    let (fb_width, fb_height) = gop_info.resolution();
    let fb_stride = gop_info.stride();

    let mut framebuffer = gop.frame_buffer();
    let fb_ptr = framebuffer.as_mut_ptr();
    let fb_size = framebuffer.size();

    // System table
    let system_table_ptr = uefi::table::system_table_raw()
        .expect("System table not initialized")
        .as_ptr() as *mut u8;

    // Memory map
    println!("Getting Memory Map and booting");

    let mmap = unsafe { uefi::boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    // UEFI Services are gone
    let mmap_ptr = mmap.buffer().as_ptr();
    let mmap_len = mmap.buffer().len();
    let mmap_desc_size = mmap.meta().desc_size;

    let boot_info = BootInfo {
        fb_ptr,
        fb_size,
        fb_width,
        fb_height,
        fb_stride,

        mmap_ptr,
        mmap_len,
        mmap_desc_size,

        system_table_ptr,
    };

    boot::stall(Duration::from_secs(10));

    Status::SUCCESS
}
