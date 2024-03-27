use crate::interface::CpuInterruptLib;

pub struct CpuInterruptLibX64;

impl CpuInterruptLib for CpuInterruptLibX64 {
    fn init() {
        // Do nothing
    }
}

pub struct CpuInterruptLibStd;
impl CpuInterruptLib for CpuInterruptLibStd {
    fn init() {
        // Do nothing
    }
}
