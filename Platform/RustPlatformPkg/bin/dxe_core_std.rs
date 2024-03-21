extern crate alloc;
use pkg1::component::DxeCoreComponent;

use uefi_core::{component, Component};

type Driver = component!(
    DxeCoreComponent<DebugLib, CpuInterrupt>;
    DebugLib=pkg1::library::DebugLibStd;
    CpuInterrupt=pkg1::library::CpuInterruptStd;
);

fn main() -> uefi_core::error::Result<()> {
    Driver::entry_point(std::ptr::null_mut(), std::ptr::null_mut())
}
