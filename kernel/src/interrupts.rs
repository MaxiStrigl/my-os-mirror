use core::{fmt, marker::PhantomData};

use spin::LazyLock;

use crate::gdt;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct InterruptDescriptor<F> {
    offset_low: u16,
    segment_selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<F> InterruptDescriptor<F> {
    pub fn new() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            ist: 0,
            type_attributes: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
            phantom: PhantomData,
        }
    }

    pub fn set_handler(&mut self, addr: u64, code_selector: u16) {
        self.offset_low = (addr) as u16;
        self.offset_mid = ((addr >> 16) & 0xFFFF) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.segment_selector = code_selector;
        self.type_attributes = 0b10001110;
    }

    pub fn set_ist(&mut self, index: u8) {
        assert!(index < 8, "IST must be between 0 and 7");
        self.ist = index;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#018x}", self.0)
            }
        }

        f.debug_struct("InterruptStackFrame")
            .field("instruction_pointer", &Hex(self.instruction_pointer))
            .field("code_segment", &Hex(self.code_segment))
            .field("cpu_flags", &Hex(self.cpu_flags))
            .field("stack_pointer", &Hex(self.stack_pointer))
            .field("stack_segment", &Hex(self.stack_segment))
            .finish()
    }
}

type HandlerFunc = extern "x86-interrupt" fn(&mut InterruptStackFrame);
type HandlerFuncWithErrorCode =
    extern "x86-interrupt" fn(&mut InterruptStackFrame, error_code: u64);
type DivergingHandlerFuncWithErrorCode =
    extern "x86-interrupt" fn(&mut InterruptStackFrame, error_code: u64) -> !;

impl InterruptDescriptor<HandlerFunc> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: HandlerFunc, code_selector: u16) {
        let addr = unsafe { core::mem::transmute_copy::<HandlerFunc, u64>(&handler) };

        self.set_handler(addr, code_selector);
    }
}

impl InterruptDescriptor<HandlerFuncWithErrorCode> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: HandlerFuncWithErrorCode, code_selector: u16) {
        let addr = unsafe { core::mem::transmute_copy::<HandlerFuncWithErrorCode, u64>(&handler) };

        self.set_handler(addr, code_selector);
    }
}
impl InterruptDescriptor<DivergingHandlerFuncWithErrorCode> {
    #[inline]
    pub fn set_handler_fn(
        &mut self,
        handler: DivergingHandlerFuncWithErrorCode,
        code_selector: u16,
    ) {
        let addr = unsafe {
            core::mem::transmute_copy::<DivergingHandlerFuncWithErrorCode, u64>(&handler)
        };

        self.set_handler(addr, code_selector);
    }
}

#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: InterruptDescriptor<HandlerFunc>,
    pub debug: InterruptDescriptor<HandlerFunc>,
    pub non_maskable_interrupt: InterruptDescriptor<HandlerFunc>,
    pub breakpoint: InterruptDescriptor<HandlerFunc>,
    pub overflow: InterruptDescriptor<HandlerFunc>,
    pub bound_range_exceeded: InterruptDescriptor<HandlerFunc>,
    pub invalid_opcode: InterruptDescriptor<HandlerFunc>,
    pub device_not_available: InterruptDescriptor<HandlerFunc>,
    pub double_fault: InterruptDescriptor<DivergingHandlerFuncWithErrorCode>,
    coprocessor_segment_overrun: InterruptDescriptor<HandlerFunc>,
    pub invalid_tss: InterruptDescriptor<HandlerFuncWithErrorCode>,
    pub segment_not_present: InterruptDescriptor<HandlerFuncWithErrorCode>,
    pub stack_segment_fault: InterruptDescriptor<HandlerFuncWithErrorCode>,
    pub general_protection_fault: InterruptDescriptor<HandlerFuncWithErrorCode>,
    pub page_fault: InterruptDescriptor<HandlerFuncWithErrorCode>,
    reserved: InterruptDescriptor<HandlerFunc>,
    pub x87_floating_point: InterruptDescriptor<HandlerFunc>,
    pub alignment_check: InterruptDescriptor<HandlerFuncWithErrorCode>,
    pub machine_check: InterruptDescriptor<HandlerFunc>,
    pub simd_floating_point: InterruptDescriptor<HandlerFunc>,
    pub virtualization: InterruptDescriptor<HandlerFunc>,
    pub control_protection: InterruptDescriptor<HandlerFuncWithErrorCode>,
    reserved2: [InterruptDescriptor<HandlerFunc>; 10],
    pub interrupts: [InterruptDescriptor<HandlerFunc>; 256 - 22 - 10],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            divide_error: InterruptDescriptor::new(),
            debug: InterruptDescriptor::new(),
            non_maskable_interrupt: InterruptDescriptor::new(),
            breakpoint: InterruptDescriptor::new(),
            overflow: InterruptDescriptor::new(),
            bound_range_exceeded: InterruptDescriptor::new(),
            invalid_opcode: InterruptDescriptor::new(),
            device_not_available: InterruptDescriptor::new(),
            double_fault: InterruptDescriptor::new(),
            coprocessor_segment_overrun: InterruptDescriptor::new(),
            invalid_tss: InterruptDescriptor::new(),
            segment_not_present: InterruptDescriptor::new(),
            stack_segment_fault: InterruptDescriptor::new(),
            general_protection_fault: InterruptDescriptor::new(),
            page_fault: InterruptDescriptor::new(),
            reserved: InterruptDescriptor::new(),
            x87_floating_point: InterruptDescriptor::new(),
            alignment_check: InterruptDescriptor::new(),
            machine_check: InterruptDescriptor::new(),
            simd_floating_point: InterruptDescriptor::new(),
            virtualization: InterruptDescriptor::new(),
            control_protection: InterruptDescriptor::new(),
            reserved2: [InterruptDescriptor::new(); 10],
            interrupts: [InterruptDescriptor::new(); 256 - 22 - 10],
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    pub size: u16,
    pub offset: u64,
}

impl InterruptDescriptorTable {
    pub fn load(&self) {
        let ptr = IdtPointer {
            size: (core::mem::size_of::<Self>() - 1) as u16,
            offset: self as *const _ as u64,
        };
        unsafe { core::arch::asm!("lidt [{}]", in(reg) &ptr, options(nostack, preserves_flags)) }
    }
}

pub static IDT: LazyLock<InterruptDescriptorTable> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    let code_selector = gdt::get_code_selector();

    idt.divide_error
        .set_handler_fn(divide_by_zero_handler as HandlerFunc, code_selector);

    idt.general_protection_fault.set_handler_fn(
        general_protection_fault_handler as HandlerFuncWithErrorCode,
        code_selector,
    );

    idt.page_fault.set_handler_fn(
        page_fault_handler as HandlerFuncWithErrorCode,
        code_selector,
    );

    idt
});

pub fn init() {
    IDT.load();

    crate::serial_println!("IDT loaded");
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut InterruptStackFrame) {
    crate::serial_println!("EXCEPTION: Divide by zero");
    crate::serial_println!("{:#?}", stack_frame);
    panic!("KERNEL PANIC: Divide by zero")
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    crate::serial_println!("EXCEPTION: General protection fault");
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("{:#?}", stack_frame);
    panic!("KERNEL PANIC: GPF")
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nostack, preserves_flags, nomem));
    }

    crate::serial_println!("EXCEPTION: Page fault");
    crate::serial_println!("Accessed Address (CR2): {:#x}", cr2);
    crate::serial_println!("Error Code: {:#x}", error_code);
    crate::serial_println!("{:#?}", stack_frame);
    panic!("KERNEL PANIC: Page fault")
}
