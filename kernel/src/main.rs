#![no_std]
#![no_main]

mod serial;

use common::BootInfo;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "C" fn _start(info: &'static BootInfo) -> ! {
    serial::inti_serial();

    let world = "World";

    serial_println!("Hello {}", world);

    let dummy: u64 = 0;
    let stack_pointer = &dummy as *const _ as u64;

    let is_higher_half = (stack_pointer & (1 << 63)) != 0;

    let color = if is_higher_half {
        0x0000FF00
    } else {
        0x00FF0000
    };

    let mut first = true;
    loop {
        if first {
            first = false;

            let fb_ptr = info.fb_ptr as *mut u32;

            for y in 0..info.fb_height {
                for x in 0..info.fb_width {
                    let pixel_offset = y * info.fb_stride + x;

                    unsafe {
                        *fb_ptr.add(pixel_offset) = color;
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
