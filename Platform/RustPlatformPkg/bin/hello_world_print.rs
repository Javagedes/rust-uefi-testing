#![no_std]
#![no_main]

extern crate alloc;
use core::panic::PanicInfo;
use r_efi::efi;

use pkg1::component::HelloWorldComponent;

use uefi_core::{Component, component};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

type Driver = component!(
    HelloWorldComponent<DebugLib>;
    DebugLib=pkg1::library::DebugLibBase
);


#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    match Driver::entry_point(image_handle, system_table) {
        Ok(..) => efi::Status::SUCCESS,
        Err(e) => e.into()
    }
}
