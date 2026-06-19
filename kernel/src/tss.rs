#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilage_stack_table: [u64; 3], // RSP
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7], // IST
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub fn new() -> Self {
        Self {
            reserved_1: 0,
            privilage_stack_table: [0; 3],
            reserved_2: 0,
            interrupt_stack_table: [0; 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: core::mem::size_of::<TaskStateSegment>() as u16,
        }
    }
}
