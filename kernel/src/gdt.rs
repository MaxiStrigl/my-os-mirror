use spin::Lazy;

pub static GDT: Lazy<(GlobalDescriptorTable, SegmentSelectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code());
    let data_selector = gdt.add_entry(Descriptor::kernel_data());

    (
        gdt,
        SegmentSelectors {
            code_selector,
            data_selector,
            tss_selector: None,
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
    }
}
