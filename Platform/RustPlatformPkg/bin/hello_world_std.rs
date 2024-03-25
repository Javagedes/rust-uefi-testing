extern crate alloc;
use pkg1::component::HelloWorldComponent;

use mu_core::{component, Component};

type Driver = component!(
    HelloWorldComponent<DebugLib>;
    DebugLib=pkg1::library::DebugLibStd;
);

fn main() -> mu_core::error::Result<()> {
    Driver::entry_point(std::ptr::null_mut(), std::ptr::null_mut())
}
