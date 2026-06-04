#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;
use common::BootInfo;
use uefi::{
    boot::{
        AllocateType::{self, AnyPages},
        MemoryType,
    },
    mem::memory_map::MemoryMap,
    prelude::*,
    println,
    proto::{
        console::gop::GraphicsOutput,
        media::file::{File, FileAttribute, FileInfo, FileMode::Read},
    },
};

use xmas_elf::{ElfFile, program::Type};

const PAGE_PRESENT: u64 = 1;
const PAGE_WRITEABLE: u64 = 1 << 1;
const PAGE_HUGE: u64 = 1 << 7;
const TABLE_FLAGS: u64 = PAGE_WRITEABLE | PAGE_PRESENT;
const HUGE_PAGE_FLAGS: u64 = PAGE_HUGE | PAGE_WRITEABLE | PAGE_PRESENT;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    pub fn new() -> &'static mut Self {
        let ptr = uefi::boot::allocate_pages(AnyPages, uefi::boot::MemoryType::LOADER_DATA, 1)
            .expect("Failed to allocate page  table")
            .as_ptr() as *mut PageTable;

        unsafe {
            core::ptr::write_bytes(ptr, 0, 1);
            &mut *ptr
        }
    }
}

fn setup_higher_half_paging(kernel_phys_addr: u64, kernel_size: u64) -> u64 {
    let pml4 = PageTable::new();
    let pdpt_ident = PageTable::new();

    // Identity map the lowest 4 GiB for the bootloader to continue to work
    pml4.entries[0] = (pdpt_ident as *const _ as u64) | TABLE_FLAGS;
    for i in 0..4 {
        let pd_ident = PageTable::new();
        pdpt_ident.entries[i] = (pd_ident as *const _ as u64) | TABLE_FLAGS;
        for j in 0..512 {
            let phys_addr = (i as u64 * 0x4000_0000) + (j as u64 * 0x200000);
            pd_ident.entries[j] = phys_addr | HUGE_PAGE_FLAGS;
        }
    }

    // Map Higher half
    // Virtual Address: 0xFFFFFFFF80000000
    // PML4 Index = 511
    // PDPT Index = 510
    // PD Index = 0

    let pdpt_high = PageTable::new();
    let pd_high = PageTable::new();

    pml4.entries[511] = (pdpt_high as *const _ as u64) | TABLE_FLAGS;
    pdpt_high.entries[510] = (pd_high as *const _ as u64) | TABLE_FLAGS;

    let num_pt_entries = (kernel_size + 0xFFF) / 0x1000;
    let num_pts = (num_pt_entries + 511) / 512;

    for i in 0..num_pts {
        let pt = PageTable::new();
        pd_high.entries[i as usize] = (pt as *const _ as u64) | TABLE_FLAGS;
        for j in 0..512 {
            let page_idx = i * 512 + j;
            if page_idx < num_pt_entries {
                let offset = page_idx * 0x1000;
                pt.entries[j as usize] = (kernel_phys_addr + offset) | TABLE_FLAGS;
            }
        }
    }

    pml4 as *const _ as u64
}

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

    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = 0_u64;

    for ph in elf.program_iter() {
        if ph.get_type() == Ok(Type::Load) {
            let start = ph.virtual_addr();
            let end = start + ph.mem_size();
            if start < min_vaddr {
                min_vaddr = start;
            }
            if end > max_vaddr {
                max_vaddr = end;
            }
        }
    }

    let kernel_mem_size = max_vaddr - min_vaddr;
    let pages_needed = ((kernel_mem_size + 0xFFF) / 0x1000) as usize;

    let kernel_phys_ptr = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages_needed,
    )
    .expect("Failed to allocate memory pages for kernel")
    .as_ptr();

    let kernel_phys_start = kernel_phys_ptr as u64;

    for ph in elf.program_iter() {
        if ph.get_type() == Ok(Type::Load) {
            let offset = ph.virtual_addr() - min_vaddr;
            let dest = (kernel_phys_start + offset) as *mut u8;
            let file_size = ph.file_size() as usize;
            let mem_size = ph.mem_size() as usize;
            let file_offset = ph.offset() as usize;

            unsafe {
                core::ptr::write_bytes(dest, 0, mem_size);

                core::ptr::copy_nonoverlapping(
                    kernel_buffer[file_offset..file_offset + file_size].as_ptr(),
                    dest,
                    file_size,
                );
            }
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

    let pml4_addr = setup_higher_half_paging(kernel_phys_start, kernel_mem_size);

    // Memory map
    // println!("Getting Memory Map and booting");

    let mmap = unsafe { uefi::boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    unsafe {
        // let pml4_addr = pml4 as *const _ as u64;
        core::arch::asm!(
          "mov cr3, {}",
          in(reg) pml4_addr,
          options(nostack, preserves_flags)
        )
    }

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
