#![no_std]
#![no_main]

extern crate alloc;
use core::panic::PanicInfo;
use r_efi::efi;

use pkg1::component::DxeCoreComponent;

use uefi_core::{Component, component};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

type Driver = component!(
    DxeCoreComponent<DebugLib, CpuInterrupt>;
    DebugLib=pkg2::library::RingBufferDebugLib;
    CpuInterrupt=pkg2::library::CpuInterruptStd;
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
