use spin::LazyLock;

use crate::tss::TaskStateSegment;

pub static TSS: LazyLock<TaskStateSegment> = LazyLock::new(|| {
    let tss = TaskStateSegment::new();
    tss
});

pub static GDT: LazyLock<(GlobalDescriptorTable, SegmentSelectors)> = LazyLock::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code());
    let data_selector = gdt.add_entry(Descriptor::kernel_data());
    let tss_selector = gdt.add_tss(&TSS);

    (
        gdt,
        SegmentSelectors {
            code_selector,
            data_selector,
            tss_selector: Some(tss_selector),
        },
    )
});

/// A single GDT entry. It is a single transparent u64 Integer
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Descriptor(u64);

impl Descriptor {
    pub fn zero() -> Self {
        Descriptor(0)
    }

    pub const fn kernel_code() -> Self {
        Descriptor(0x9A << 40 | 0xA << 52)
    }
    pub const fn kernel_data() -> Self {
        Descriptor(0x92 << 40 | 0xC << 52)
    }
    pub const fn user_code() -> Self {
        Descriptor(0xFA << 40 | 0xA << 52)
    }
    pub const fn user_data() -> Self {
        Descriptor(0xF2 << 40 | 0xC << 52)
    }

    pub fn tss(tss_ptr: &'static TaskStateSegment) -> (Self, Self) {
        let ptr = tss_ptr as *const _ as u64;
        let limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u64;

        let base_low = ptr & 0xFFFF;
        let base_mid = (ptr >> 16) & 0xFF;
        let base_high = (ptr >> 24) & 0xFF;
        let base_upper = (ptr >> 32);

        let type_tss = 0b1001 << 40;
        let present = 1 << 47;

        // lower desscriptor is a normal segmetn
        let lower = (limit & 0xFFFF)
            | (base_low << 16)
            | (base_mid << 32)
            | type_tss
            | present
            | (base_high << 56);

        // Upper 64 just contain the remaining 32 bits of base
        let higher = base_upper;

        return (Descriptor(lower), Descriptor(higher));
    }
}

pub struct SegmentSelectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: Option<SegmentSelector>,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct SegmentSelector(u16);

impl SegmentSelector {
    pub const fn new(index: u16, rpl: u16) -> Self {
        SegmentSelector((index << 3) | (rpl & 0b11))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

pub struct GlobalDescriptorTable {
    entries: [Descriptor; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    fn new() -> Self {
        GlobalDescriptorTable {
            entries: [Descriptor::zero(); 8],
            next_free: 1,
        }
    }

    fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = self.next_free;
        assert!(index < self.entries.len(), "GDT is full!");

        self.entries[index] = entry;
        self.next_free += 1;

        let rpl = (((entry.0) >> 45) & 0b11) as u16;
        SegmentSelector::new(index as u16, rpl)
    }

    pub fn load(&'static self) {
        let ptr = GdtPointer {
            limit: ((self.next_free * core::mem::size_of::<Descriptor>()) - 1) as u16,
            base: self.entries.as_ptr() as u64,
        };

        unsafe {
            use core::arch::asm;
            asm!("lgdt [{}]", in(reg) &ptr, options(nostack, preserves_flags, readonly));
        }
    }

    fn add_tss(&mut self, tss: &'static TaskStateSegment) -> SegmentSelector {
        let index = self.next_free;
        assert!(index + 1 < self.entries.len(), "GDT is full!");

        let (low, high) = Descriptor::tss(tss);
        self.entries[index] = low;
        self.entries[index + 1] = high;
        self.next_free += 2;

        SegmentSelector::new(index as u16, 0) //  Ring 0 RPL
    }
}

pub fn init() {
    GDT.0.load();
    unsafe {
        // reload code segment selector
        let cs = GDT.1.code_selector.0 as u64;
        core::arch::asm!(
        "push {sel}",
        "lea {tmp}, [rip + 2f]",
        "push {tmp}",
        "retfq",
        "2:",
        sel = in(reg) cs,
        tmp = lateout(reg) _,
        );

        // reload data segment selector
        let ds = GDT.1.data_selector.0;
        core::arch::asm!(
        "mov ds, {0:x}",
        "mov es, {0:x}",
        "mov fs, {0:x}",
        "mov gs, {0:x}",
        "mov ss, {0:x}",
        in(reg) ds,
        );

        if let Some(tss_selector) = GDT.1.tss_selector {
            core::arch::asm!("ltr ax", in("ax") tss_selector.0, options(nostack, preserves_flags));
        }
    }
}

pub fn read_task_register() -> u16 {
    let tr: u16;
    unsafe {
        core::arch::asm!("str {0:x}", out(reg) tr, options(nomem, nostack, preserves_flags));
    }

    tr
}
