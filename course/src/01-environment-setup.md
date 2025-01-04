# Environment setup

## Supply List 
I have split this list into two categories: the essential items are the bare minimum to get anything running using this repository and the nonessentials are nice-to-haves that give you the leeway to continue tinkering on your own after this course is complete.

### Essential Supplies
- Raspberry Pi Pico development board with headers - [Mouser # 358-SC0917](https://www.mouser.com/ProductDetail/358-SC0917)
  - The board with included headers is worth the added expense because it has a socket that directly plugs into the debug probe without any soldering required.
- Raspberry Pi Debug Probe - [Mouser # 358-SC0889](https://www.mouser.com/ProductDetail/358-SC0889)
  - There are ways to get code to flash to a Raspberry Pi Pico without a debug probe but having one in your toolbox is great.

### Nonessential Supplies
- Breadboard power supply - [Mouser # 474-PRT-21297](https://www.mouser.com/ProductDetail/474-PRT-21297)
  - Barrel jack adapter for power - [Mouser # 474-TOL-15313](https://www.mouser.com/ProductDetail/474-TOL-15313)
- Breadboards - [Mouser # 589-TW-E40-1020](https://www.mouser.com/ProductDetail/589-TW-E40-1020)
  - Note the variation in quality for breadboards is high, these will likely be fine but if you want the best of the best look [here](https://www.assemblyspecialist.com/WebStore/breadboards.html), they are the supplier for 3M which are widely accepted as being the best breadboards around, but very expensive.
- Breadboard jumper wire kit - [Mouser # 474-PRT-00124](https://www.mouser.com/ProductDetail/474-PRT-00124)

## Development Environment
### OS 
 - Most modern Linux distros will be fine for this. I used a freshly downloaded Debian 12 VM.
 - WSL has a hard time interfacing with hardware devices; a VM is better for this project.
 - MacOS generally mirrors Linux using the HomeBrew package manager.
 - Windows is untested, attempt at your peril.
### Rust
- If you don't have Rust installed already you will need to install `rustup`, the Rust toolchain manager.
- Follow the install directions [here](https://rustup.rs/)
- Once complete you should be able to use the `cargo` and `rustup` commands
### Probe-rs
- Probe-rs is a collection of tools to aid with embedded development in Rust, it includes software to interface with debug probes and integrations with Rust's build system, Cargo. 
- Install probe-rs following the websites directions:
  - [Main site](https://probe.rs/)
  - [Installation page](https://probe.rs/docs/getting-started/installation/)
  - 🚨 If you are on Linux ensure you add a udev rule to allow non-root users to access debug probes, directions [here](https://probe.rs/docs/getting-started/probe-setup/#linux%3A-udev-rules)
### IDE
- See the official [Rust tooling](https://www.rust-lang.org/tools) page
- For people new to Rust or if you have no real preference, I would recommend VSCode or Neovim with the rust-analyzer LSP.

## Setup verification
### Raspberry Pi Pico
Before we move on to trying to program a board, we should always check that the manufacturer examples work. This will cut down on the scope of troubleshooting needed when something breaks.
- Follow the guide [here](https://github.com/raspberrypi/documentation/blob/develop/documentation/asciidoc/microcontrollers/c_sdk/your_first_binary.adoc) 
- This guide will walk you through loading a blinking light firmware on to your board through its native USB interface.
- The debug probe is not required for this step.

### Debug Probe Flashing
Now that we know the Pi Pico is running, it is time to push some example code through the debug probe interface. 
- Plug in your debug probe to the Pi Pico
- Note that this connection does not supply power to the Pico, either plug it in via USB or an external power supply. 
- Regardless of the method you choose to power the board, light should still be blinking from the last section.
- Pull [this repository](https://github.com/rp-rs/rp2040-project-template) to your development machine
- Install some build tools:
  - `rustup target add thumbv6m-none-eabi`
  - `cargo install flip-link`
  - `sudo apt install gcc`
  - 🚨 Ensure you have configured the UDEV rules mentioned [here](https://probe.rs/docs/getting-started/probe-setup/#linux%3A-udev-rules)
- Run `cargo run`
- You should see a successful code flash and the print statements from the debug interface being printed to your console

