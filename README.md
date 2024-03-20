## uefi_core

This crate provides the trait definition for a Component, and a error enum for converting between the typical rust error handling (with "?"s) and UEFI error handling (returning EFI_X)

## uefi_macro

This crate provides the component!() macro for generating the type definition for a component.

## RustPkg1

This crate contains the library trait for DebugLib, a few implementations of the library, and a component, HelloWorld.

## RustPkg2

This crate contains a library implementation for DebugLib

## RustPlatformPkg

This crate contains component implementations in the bin/* folder. i.e. they get compiled into binaries, and are where you specify which library implementations you want to use. 