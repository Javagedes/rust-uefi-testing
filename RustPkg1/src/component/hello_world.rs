use core::marker::PhantomData;
use log::info;
use r_efi::efi;

use uefi_core::{Component, error::Result};
use crate::interface::DebugLib;

pub struct HelloWorldComponent<D>
where
    D: DebugLib
{
    _d: PhantomData<D>,
}


impl <D> Component for HelloWorldComponent<D>
where
    D: DebugLib,
{
    fn main(_: efi::Handle, _: *mut efi::SystemTable) -> Result<()>{
        info!("Hello, World! (With Love, From Joey)");
        info!("Writing some more bytes my dude");
        Ok(())
    }

    fn init(ih: efi::Handle, st: *mut efi::SystemTable) -> Result<()>{
        D::init(ih, st);
        Ok(())
    }
}