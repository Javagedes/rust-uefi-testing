mod debug_lib;
mod cpu_interrupt;

pub use debug_lib::DebugLibBase;
pub use debug_lib::DebugLibNull;
pub use cpu_interrupt::CpuInterruptLibX64;

#[cfg(feature = "std")]
pub use debug_lib::with_std::DebugLibStd;
#[cfg(feature = "std")]
pub use cpu_interrupt::CpuInterruptLibStd;