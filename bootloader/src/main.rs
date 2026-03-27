#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;
use common::BootInfo;
use uefi::{
    boot::{AllocateType, MemoryType},
    mem::memory_map::MemoryMap,
    prelude::*,
    println,
    proto::{
        console::gop::GraphicsOutput,
        media::file::{File, FileAttribute, FileInfo, FileMode::Read},
    },
};

use xmas_elf::{
    ElfFile,
    program::{ProgramHeader, Type},
};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    println!("Bootloader initialized");

    let mut sfs = boot::get_image_file_system(boot::image_handle()).expect("No SFS");

    let mut root = sfs.open_volume().expect("Failed to open sfs");

    let mut file_handle = root
        .open(cstr16!("kernel.elf"), Read, FileAttribute::empty())
        .expect("Kernel not found")
        .into_regular_file()
        .expect("msg");

    println!("Kernel found. Loading into file buffer");

    let mut buffer = [0u8; 128];
    let file_info = file_handle.get_info::<FileInfo>(&mut buffer).unwrap();
    let mut kernel_buffer = vec![0u8; file_info.file_size() as usize];
    file_handle
        .read(&mut kernel_buffer[..])
        .expect("Failed to read kernel");

    let elf = ElfFile::new(&kernel_buffer[..]).expect("Invalid ELF");

    for ph in elf.program_iter() {
        if ph.get_type() == Ok(Type::Load) {
            load_segment(&kernel_buffer[..], ph);
        }
    }

    let entry = elf.header.pt2.entry_point();

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

    let kernel_entry: extern "C" fn(&'static BootInfo) -> ! =
        unsafe { core::mem::transmute(entry) };

    kernel_entry(unsafe { core::mem::transmute(&boot_info) });
}

fn load_segment(kernel_bytes: &[u8], ph: ProgramHeader) {
    let start_addr = ph.physical_addr();
    let mem_size = ph.mem_size();
    let file_size = ph.file_size();
    let offset = ph.offset() as usize;

    let page_count = ((mem_size + 0xfff) / 0x1000) as usize;

    boot::allocate_pages(
        AllocateType::Address(start_addr),
        MemoryType::LOADER_DATA,
        page_count,
    )
    .expect("Failed to allocate pages");

    let dest = start_addr as *mut u8;

    unsafe {
        // Zero out
        core::ptr::write_bytes(dest, 0, mem_size as usize);

        // Copy
        core::ptr::copy_nonoverlapping(
            kernel_bytes[offset..offset + file_size as usize].as_ptr(),
            dest,
            file_size as usize,
        );
    }
}
