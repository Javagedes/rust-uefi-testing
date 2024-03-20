#![no_std]
pub mod error;

use r_efi::efi;

pub use uefi_macro::component;

pub trait Component {
    fn main(
        image_handle: efi::Handle,
        system_table: *mut efi::SystemTable,
    ) -> error::Result<()>;

    fn init(
        image_handle: efi::Handle,
        system_table: *mut efi::SystemTable,
    ) -> error::Result<()>;

    fn entry_point(
        image_handle: efi::Handle,
        system_table: *mut efi::SystemTable,
    ) -> error::Result<()> {
        rust_boot_services_allocator_dxe::GLOBAL_ALLOCATOR.init(unsafe { (*system_table).boot_services });
        
        Self::init(image_handle, system_table)?;
        Self::main(image_handle, system_table)
    }
}