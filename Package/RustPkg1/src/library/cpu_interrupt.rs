use crate::interface::CpuInterrupt;

pub struct CpuInterruptX64;

impl CpuInterrupt for CpuInterruptX64 {
    fn init() {
        // Do nothing
    }
}

pub struct CpuInterruptStd;
impl CpuInterrupt for CpuInterruptStd {
    fn init() {
        // Do nothing
    }
}
