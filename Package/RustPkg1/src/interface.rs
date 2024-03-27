use r_efi::efi::{Handle, SystemTable};

/// A Trait for a Rust-UEFI debugging library that use's the crate `log`'s macros.
pub trait DebugLib {
    fn init(image_handle: Handle, system_table: *mut SystemTable);
}

pub trait CpuInterruptLib {
    fn init();
}
