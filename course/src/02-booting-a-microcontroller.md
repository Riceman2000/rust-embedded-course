# Booting a Microcontroller

# Creating the Binary

In the README, we learned about flashing code onto the microcontroller using a debug probe. That is great, but how do we know that our code is going to run on the selected processor? When a program is compiled for as an application on a PC, it will will most likely use a 64 bit architecture (either x64 or ARM). Normally the architecture is selected for the programmer and it is not something that must be considered, tools generally assume that you are building for the system that you are currently using. However, when programming a microcontroller, the architecture will not be the same. Using a significantly simplified architecture comes with unique challenges but also benefits in power efficiency. This chapter will outline finding the specific instruction set for a microcontroller and the configuration of a development environment to compile to it along with the consequences of a resource limited system. 

## The `no_std` and `no_main` Environment

The Rust [standard library](https://doc.rust-lang.org/std/) or just `std` contains many useful tools for everyday development. However, it is difficult to implement all those tools on limited systems. Generally, the starting point for embedded Rust is in the `no_std` environment. By default, Rust includes the standard library in program files, meaning you can just use `Vec::new()` instead of having to fully specify `std::vec::Vec::new()` or use a `use` statement. This behavior is disabled by using the `#[no_std]` attribute in the `main.rs` file.

Note, you can still use standard library functions in the `no_std` environment, however, you will have to explicitly include the desired code and manually configure certain behavior. For example, to use types like `String` or `Vec` requires that heap allocation is configured using a [global allocator](https://doc.rust-lang.org/std/alloc/). In other words, `no_std` code has _no heap allocation_ as a starting point. 

In addition to having no standard library, there is also no provision for a `main` function. Instead, we need to properly place our `entry` function in memory so that the CPU begins execution at the beginning of our program. Moreover, the processor needs certain things to be done before we can start executing general-purpose code, such as setting up external flash memory and initializing registers. By default, Rust configures a `main` function for the programmer for whatever target is selected, however, this is not the case for embedded systems. This behavior is disabled by using the `#[no_main]` attribute in the `main.rs` file.

If you are curious about what a `no_main` Rust file looks like in a familiar environment, take a look at [this example](https://github.com/johnthagen/min-sized-rust/blob/main/no_main/nix/src/main.rs) of a Rust program written without a `main` function for a UNIX system. 

The first two lines of our minimal program will be just the two attributes discussed above:

`src/main.rs`
```rust
#![no_std]
#![no_main]
```

## Cross Compilation

In Rust, we specify a cross-compilation target via a "target triple." The target triple is a string that fully defines what the output byte code should look like for whatever processor we select. The Rust [platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) page has an index of the available targets with their associated level of support.

### Target Selection

So, how do we select a target? The target triple is made up of a few fields [defined here](https://docs.rs/target-lexicon/latest/target_lexicon/struct.Triple.html). In our case, we need to know what processor we are working with. This is the first of many times we are going to look in the datasheet. Open this and bookmark it because we are going to be using it constantly:

https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf

On page 10, the processor is listed as `Dual ARM Cortex-M0+ @ 133MHz`. The [ARM website](https://developer.arm.com/Processors/Cortex-M0-Plus) defines this processor as using the `Armv6-M` architecture with the `Thumb or Thumb-2 subset` instruction set. With that information, we can look into the platform support page to find our target. The [thumbv6m-none-eabi](https://doc.rust-lang.org/nightly/rustc/platform-support/thumbv6m-none-eabi.html) target is the correct instruction set and architecture, and it lists our processor as a supported processor.

### Compiling to a Target 

Now that we have our target `thumbv6m-none-eabi`, let's actually use it. While you could use the command line arguments for all of this, it is much more convenient to use a config file:

`.cargo/config.toml`
```toml
[build]
target = "thumbv6m-none-eabi"
```

Just compiling the program to the correct instruction set is not enough, however. We need to properly place the program data in memory. We do this through the linker.

## Understanding the Hardware

Before we move on to linking our program, we need to understand what we are working with on our dev board. Note that the `RP2040` is not the _board_ you have in front of you, but instead it is just the small chip at the center of it. There are other supporting components that facilitate it doing its job, and one of those is external flash memory. The following datasheet is for the Pi Pico itself, not the RP2040 microcontroller:

https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf

This datasheet will be useful for things such as power supply specifications, seeing what external supporting components are used, and understanding how our microcontroller pins are broken out to pins that we can actually use on our dev board.

### The Boot Sequence

Finding section 2.8.1 of the [RP2040 datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf) gives us a solid idea of what the boot sequence is going to look like. We see that the controller is going to pull 256B out of flash memory first before it enters the "flash second stage" and executes the code that was just retrieved. The first 256B are known as the second-stage boot loader, and its job is to ensure that the processor is set up to read from the external flash memory. This is necessary because there are many different options for memory chips that a designer could choose, each with slightly different ways to access their contents. 

However, you may have noticed a slight logical inconsistently, namely that the microcontroller is reading from flash memory to get the instructions it needs to read from flash memory. Section 2.8.1.2 outlines the commands that are sent via SPI to the flash memory, it is up to the designer to select a memory chip that will respond favorably to this sequence of commands. The commands selected here are the generalized and are consequently less efficient than what is possible with SPI flash memory chips, the second-stage boot loader can reconfigure the memory chip to run in its fastest configuration.

A second-stage bootloader for the Pi Pico's memory chip, the W25Q080, is openly available. The crate [`rp2040-boot2`](https://crates.io/crates/rp2040-boot2) provides that bootloader in a convenient wrapping. If you'd like to read what goes into those 256B, it is all [here](https://github.com/rp-rs/rp2040-boot2/blob/main/src/boot2_w25q080.S).

To use the `rp2040-boot2` crate, we need to include some code in our `main.rs` and `Cargo.toml` files:

`main.rs`
```rust
#![no_std]
#![no_main]

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
```

`Cargo.toml`
```toml
[package]
edition = "2021"
name = "A-minimal-flash"
version = "0.1.0"

[dependencies]
rp2040-boot2 = "0.3"
```

## Linking

By providing the [`cortex-m-rt`](https://crates.io/crates/cortex-m-rt) crate with a `memory.x` file and a few attributes in our program, it will produce a linker script that will properly place our program data for us.

### Memory Layout

The `memory.x` file needed by `cortex-m-rt` is a description of how to layout the address space of our program. The [`rp2040-boot2`](https://crates.io/crates/rp2040-boot2) crate that is needed to supply the second-stage bootloader also provides an example `memory.x` file:

`memory.x` from [`rp2040-boot2`](https://crates.io/crates/rp2040-boot2) docs
```
MEMORY
{
  /* To suit Raspberry Pi RP2040 SoC */
  BOOT_LOADER : ORIGIN = 0x10000000, LENGTH = 0x100
  FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
  RAM : ORIGIN = 0x20000000, LENGTH = 264K
}

SECTIONS {

  /* ### Boot loader */
  .boot_loader ORIGIN(BOOT_LOADER) :
  {
    KEEP(*(.boot_loader*));
  } > BOOT_LOADER

} INSERT BEFORE .text;
```

If you were wondering where these numbers came from, see section 2.2.1 in the datasheet. Making decisions about how to layout memory is a complex topic that is out of the scope of this course.

### Entry Point

The entry point of your program in an embedded environment is telling the linker that the function's code needs to be placed at whatever address will be in the `PC` (program counter) register when the initialization code is done. The `PC` register is a register internal to the CPU that holds the address of the instruction it is currently executing. As instructions are executed, the `PC` register is incremented or, in the case of some control flow, directly modified.

With the `cortex-m-rt` crate, the entry point is easy to set. The [`cortex-m`](https://crates.io/crates/cortex-m) crate can also be used to access assembly instructions for an empty busy loop.

`main.rs`
```rust
...
#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    loop {
        nop();
    }
}
```
Note: the `!` type in Rust represents the [`never`](https://doc.rust-lang.org/std/primitive.never.html) type, indicating that that type will never be realized because the function does not return.

`Cargo.toml`
```toml
...
[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
rp2040-boot2 = "0.3"
```

### Including Linker Scripts

In the Cargo config file, we need to add some configurations to ensure the linker script from `cortex-m-rt` is included:

`.cargo/config.toml`
```toml
[build]
target = "thumbv6m-none-eabi"

rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=--nmagic",
]
```

The script `link.x` is generated at compile time and changes as we update the code. `nmagic` disables page alignment, see [this](https://sourceware.org/binutils/docs-2.42/ld/Options.html) for more detail or [this](https://github.com/rust-embedded/cortex-m-quickstart/pull/95) for more Rust specific details. 

## Panic Handling

The last thing we need to do is solve this error:
```
error: `#[panic_handler]` function required, but not found
```
`Panic`ing is Rust's way of crashing a program in a controlled way. Generally, this involves unwinding the call stack to give the user a backtrace when debugging. But that doesn't mean anything on an embedded system where there is no standard output, much less a place to put logs. So what should be done in the case of a panic? In production, it may be best to send a signal to an external debug probe, trigger a processor reboot, or print some kind of message over a UART port.

However, for now, the crate [`panic-halt`](https://crates.io/crates/panic-halt) is a great option. It implements the bare minimum needed to compile with a valid panic handler. Its source code in generally just an [infinite loop with extra stuff](https://github.com/korken89/panic-halt/blob/master/src/lib.rs).

Including this panic handler looks like this in practice:

`main.rs`
```rust
...
use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
...
```

`Cargo.toml`
```toml
...
[dependencies]
...
panic-halt = "0.2"
```

# Flashing Our Code

Most of the heavy lifting for this has already been done in the first chapter. If you haven't completed chapter one, ensure you go back and follow those instructions to make sure all of your hardware and software are communicating appropriately.

## Using `probe-rs`

First, let's try to use `probe-rs` by itself by running these commands:
```sh
cargo build
probe-rs run --chip RP2040 --protocol swd target/thumbv6m-none-eabi/debug/A-minimal-flash
```
If no error messages popped up, it probably worked, though it is not easy to tell because this code doesn't do anything yet.

## Using a Cargo Runner

Typing out that command each time is cumbersome. It would be better if we could just use `cargo run` like normal. This is possible by changing what cargo uses as the `runner`.

`.cargo/config.toml`
```toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-rs run --chip RP2040 --protocol swd"

[build]
target = "thumbv6m-none-eabi"
```

This configuration specifies that any target that uses an ARM-based architecture with no OS will use this runner command. While it is out of the scope for this overview, it is possible to define multiple targets each with their own configuration. For example, if you wanted to run a QEMU instance with your code for your given architecture for emulated testing.

Now it should be possible to use this command to build and flash the code:
```sh
cargo run
```

## Build Profiles

Profiles are how Rust allows users to save certain compiler configurations for different stages of development. For example, the two default profiles are `debug` and `release`. When running `cargo run` with no other options, it automatically follows the `debug` profile, which optimizes less, does not strip symbols, and includes all runtime safety checks. If you would like to use the `release` profile, you can use `cargo run --release`. In `release` mode, optimizations take longer and most runtime checks are disabled. Configuration of these profiles is in the `Cargo.toml` file.

# Final Code for a Minimal Flash

With all of that done, you can take a look at the `A-minimal-flash` directory in this repository. This directory contains all of the work done in this section along with some other tweaks that make life easier, such as a `rust-analyzer.json` file that disables some LSP errors.

Read through the new files and feel free to ask any questions you may have about them.
