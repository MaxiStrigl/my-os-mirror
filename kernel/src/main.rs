#![no_std]
#![no_main]

use common::BootInfo;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "win64" fn _start(info: &'static BootInfo) -> ! {
    let mut first = true;
    loop {
        if first {
            first = false;

            let fb_ptr = info.fb_ptr as *mut u32;
            let purple = 0x00FF00FF;

            for y in 0..info.fb_height {
                for x in 0..info.fb_width {
                    let pixel_offset = y * info.fb_stride + x;

                    unsafe {
                        *fb_ptr.add(pixel_offset) = purple;
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
