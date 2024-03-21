extern crate alloc;
use pkg1::component::HelloWorldComponent;

use uefi_core::{component, Component};

type Driver = component!(
    HelloWorldComponent<DebugLib>;
    DebugLib=pkg1::library::DebugLibStd;
);

fn main() -> uefi_core::error::Result<()> {
    Driver::entry_point(std::ptr::null_mut(), std::ptr::null_mut())
}
