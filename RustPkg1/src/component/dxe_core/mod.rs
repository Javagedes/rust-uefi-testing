use core::marker::PhantomData;
use log::info;
use r_efi::efi;

use uefi_core::{Component, error::Result};
use crate::interface::{DebugLib, CpuInterrupt};

pub struct DxeCoreComponent<D, C>
where
    D: DebugLib,
    C: CpuInterrupt,
{
    _d: PhantomData<D>,
    _c: PhantomData<C>,
}

impl <D, C> Component for DxeCoreComponent<D, C>
where
    D: DebugLib,
    C: CpuInterrupt,
{
    fn main(_: efi::Handle, _: *mut efi::SystemTable) -> Result<()>{
        info!("Starting DXE Core...");
        Ok(())
    }

    fn init(ih: efi::Handle, st: *mut efi::SystemTable) -> Result<()>{
        D::init(ih, st);
        info!("Logger initialized.");
        C::init();
        Ok(())
    }
}
