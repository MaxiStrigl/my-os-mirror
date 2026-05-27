#![no_std]
#![no_main]

#[repr(C)]
pub struct BootInfo {
    // Framebuffer
    pub fb_ptr: *mut u8,
    pub fb_size: usize,
    pub fb_width: usize,
    pub fb_height: usize,
    pub fb_stride: usize,

    // Memory Map
    pub mmap_ptr: *const u8,
    pub mmap_len: usize,
    pub mmap_desc_size: usize,

    // System Table
    pub system_table_ptr: *const u8,
}
