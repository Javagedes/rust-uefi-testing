## Getting Started

### Commands

Read Everything below already? If not, read the below first. This is just better to have at the top

- `cargo build-std` Builds all crates with the std feature, including any std binary drivers.
- `cargo build-uefi` Builds all crates with the uefi feature, including any uefi drivers.
- `run-std` Runs the specified binary
- `test-mu` Runs all unit tests for mu_config and mu_macro

**Note**: You can / should add `--bin <bin_name>` to build or run only a single std driver.

### Actually Getting Started

This repository exists to create and showcase an architectural design for building components and
libraries in a decoupled way to allow for "hot swapping" of libraries when compiling a component
(which at this point in time is a DXE_DRIVER). This is accomplished by defining a component as a
struct that describes the library abstractions it needs:

```rust
pub struct HelloWorldComponent<D>
where
    D: DebugLib // Library Interface the component needs
{
    _d: PhantomData<D>
}
```
From there, the `Component` trait is implemented on the struct, which is where the core logic of
the component is located. Accessing library functionality is as easy as calling their static
methods as described by the trait interface:

```rust
impl <D>Component for HelloWorldComponent<D>
where
    D: DebugLib
{
    fn main(_: Handle, _: SystemTable) -> Result<()> {
        D::init()
        Ok(())
    }

    ...
}
```

As alluded to in the previous sentence, a library interface is simply a rust trait that the
instances will implement, ensuring the expected function interfaces exist, and allowing the
component to statically abstract away which library is used, until it is time to instantiate the
specific instance of the component (i.e. the specific libraries the component uses). This allows
the component to swap libraries instances without any coupling. Libraries can even have additional
library dependencies of their own!

```rust
struct MyDebugLib;
impl DebugLib on MyDebugLib {
    fn init() {
        // Do Nothing
    }
}

struct MyDebugLib<P: PortLib>;
impl <P> on MyDebugLib<P>
where
    P: PortLib
{
    fn init() {
        ...
    }
}
```

The final step is to create the file that gets compiled into a efi binary. This will either be a
`src/main.rs` file or a `bin/*.rs` file. Either way, these files get compiled into a efi binary
when using a `*-none-uefi` target. To do so, simply create a type alias for the driver and it's
selected drivers, then call the entry point:

```rust
type Driver = HelloWorldComponent<MyDebugLib>

#[no_mangle]
pub extern "efiapi" fn efi_main(
    ih: Handle,
    st: *mut SystemTable,
) -> Status {
    match Driver::entry_point(ih, st) {
        Ok(..) => Status::SUCCESS,
        Err(e) => e.into()
    }
}
```

By using these abstractions, it is actually possible to swap libraries for `std` supported
instances, and run your component on the host machine!

## Complex dependencies and Easily exchanging library instances

While the above example was simple and easy, real world components have much more complex library dependencies! If you have every build a dependency tree of a EDKII component, you will see that a low amount of top level dependencies can still result in a huge about of overall dependencies! Lets say your component has 2 dependencies, and those two also have two, and so on and so on... well you can do the math - 2^x
can be a lot!

One weakness of this architecture is that due to the complexity described above, creating the type alias can be incredibly complex, and changing
even a single library could be an effort. Lets take the following example: 

- Component: MyComponent with library dependencies on MyLib1 and MyLib2
- MyLib1 - MyLib1Impl with dependency on MyLib2 and MyLib3
- MyLib2 - MyLib2Impl with a dependency on MyLib4
- MyLib3 - MyLib3Impl
- MyLib4 - MyLib4Impl

A driver type alias for this would look like:

```rust
type Driver = MyComponent< MyLib1Impl< MyLib2Impl< MyLib4Impl >, MyLib3Impl >, MyLib2< MyLib4Impl > >
```

Don't worry, you don't need to fully get the above, heck I struggled to make sure I wrote it correctly! And that was a fairly simple example. In this example, lets say I wanted to swap MyLib2Impl to MyLib2ImplExtra, I now have to switch both occurrences of MyLib2Impl to MyLib2Extra, which will cascade down to the dependencies of MyLib2Impl. It would be a lot of work!

Because this is complex, we created a macro that does it for you! All that needs to be done is to write the component, and each library instance once, and the macro will take care of replacing libraries with their library instances:

```rust
type Driver = component!(MyComponent<MyLib1, MyLib2>;
    MyLib1 = MyLib1Impl<MyLib2, MyLib3>;
    MyLib2 = MyLib2Impl<MyLib4>
    MyLib3 = MyLib3Impl
    MyLib4 = MyLib4Impl
)
```

While this is slightly longer than doing it manually, it is much easier to (1) understand and (2) change library instances. Additionally, we are still working on relatively simple examples. The more complex it is, the more useful this macro is. Here is the above driver, but with Lib1 swapped:

```rust
type Driver = component!(MyComponent<MyLib1, MyLib2>;
    MyLib1 = MyLib1Impl2<MyLib3>;
    MyLib2 = MyLib2Impl<MyLib4>
    MyLib3 = MyLib3Impl
    MyLib4 = MyLib4Impl
)
```
With a simple change, we cascaded a change in the type alias:

```rust
type Driver = MyComponent< MyLib1Impl< MyLib2Impl< MyLib4Impl >, MyLib3Impl >, MyLib2< MyLib4Impl > >
type Driver = MyComponent< MyLib1Impl2< MyLib3Impl >, MyLib2< MyLib4Impl > >
```

## Configuring components through External Config files

Similar to how EDKII relies on a DSC to specify library usage, we too need a way to easily swap dependencies across all components. With what you've seen so far, if you wanted to swap MyLib1 from MyLib1Impl to MyLib1Impl2, you would need to go into each component's `bin/*.rs` file and update it. This is not very productive. So we've added a way to allow generic configurations across multiple components using a config file similar to a dsc. We've implemented it very simply, using the `toml` format.

Instead of passing library instances inside the `component!` macro, you can instead, provide it a file path to use. We are also considering adding
the ability to just read an environment variable that has the path, so it can easily be swapped even further.

```rust
type Driver = component!(MyComponent<MyLib1, MyLib2>; Config="/Path/To/Config.toml")
```

We will then use that configuration file to select the appropriate library instances - similar to the DSC. Here is an example Configuration file. It is a simple `<library_name> = <include_path>`:

``` toml
[libraries]
MyLib1 = "pkg1::library::MyLib1Impl<MyLib3>"
MyLib2 = "pkg2::library::MyLib2Impl<MyLib4>"
MyLib3 = "pkg1::library::MyLib3Impl"
MyLib4 = "pkg3::library::MyLib4Impl"
```

being as this is a toml config file, there are plenty of possibilities to add additional configuration possibilities to help mirror the functionality of DSCs. You can also note that since there really is no equivalent to an INF, we need to describe each library's library dependencies directly in this file.

## Crates

Below are the list of crates and their purpose / contents.

### mu_core

This crate provides the trait definition for a Component, and a error enum for converting between
the typical rust error handling (with "?"s) and UEFI error handling (returning EFI_X)

### mu_macro

This crate provides the component!() macro for generating the type definition for a component.

### mu_config

This crate provides an interface for parsing the config file for specifying dependencies.

### Package/RustPkg1

This crate contains the library trait for DebugLib, a few implementations of the library, and a
component, HelloWorld.

### Package/RustPkg2

This crate contains a library implementation for DebugLib

### Platform/RustPlatformPkg

This crate contains component implementations in the bin/* folder. i.e. they get compiled into
binaries, and are where you specify which library implementations you want to use. 

## RustBootServicesAllocatorDxe

A clone of https://github.com/microsoft/mu_plus/tree/release/202311/MsCorePkg/Crates/RustBootServicesAllocatorDxe