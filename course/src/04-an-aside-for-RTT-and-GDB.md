# An Aside For RTT And GDB

# RTT - Real Time Transfer
The output you have been seeing on the console as you are running the previous examples is read from the microcontroller using RTT. RTT is a protocol developed by Segger to aid in debugging microcontrollers without the need to use up a peripheral such as UART. RTT works by allocating a circular buffer in RAM that is constantly being read by the debug probe. In order to write a string into RTT all the microcontroller has to do is copy the log string to that region of RAM which is orders of magnitude faster than passing it through a UART peripheral.

## Printing to RTT
In a practical sense, printing to RTT is the same as most other debug methods. In Rust, the [`defmt`](https://crates.io/crates/defmt) crate is used to give users a standardized experience that mimics the standard [`log`](https://crates.io/crates/log) and [`tracing`](https://crates.io/crates/tracing) format. 

Using RTT in our program is as simple as calling one of the log level macros such as

`main.rs`
```rust
...
info!("Hello from RP2040!");
...
```

## Configuring RTT
The astute among you may have noticed that I mentioned a region of RAM in the explantation earlier. Any time we need to have a region of memory that both the microcontroller and the programming machine know about, some kind of configuration must be done. In this case the `defmt` crate generates a linker script. 

`.cargo/config.toml`
```toml
rustflags = [
...
  "-C", "link-arg=-Tdefmt.x",
...
]
```

Another configuration that can be done is to determine which log level should be printed. This generally defaults to `info` for most applications.

`.cargo/config.toml`
```toml
...
[env]
DEFMT_LOG = "trace"
...
```
and 

`Embed.toml`
```toml
[default.general]
chip = "RP2040"
log_level = "WARN"
# RP2040 does not support connect_under_reset
connect_under_reset = false

[default.rtt]
enabled = false
up_mode = "NoBlockSkip"
timeout = 3000
```
Note that whatever level you define in `.cargo/config.toml` is the lowest level that will be compiled into the program. The level defined in `Embed.toml` is the level that will be printed by the local RTT client. This can give the developer to include verbose logging that may adversely affect runtime performance that is only enabled when it is needed. The log levels from most verbose to least are as follows:
- `TRACE`
- `DEBUG`
- `INFO`
- `WARN`
- `ERROR`

# GDB - GNU Debugger

To get started with GDB you will need to download GDB for the architecture we are targeting. In the case of ARM Thumb v6 it can be done with:

```sh
sudo apt install gdb-multiarch
```

We also need a local proxy for GDB to bind to, which `cargo-embed` can do if configured properly. We also need to set the reset behavior to halt so the controller will not start executing code before the debugger is attached.

`Embed.toml`
```toml
...
[default.reset]
enabled = true
halt_afterwards = true
...
[default.gdb]
enabled = true
gdb_connection_string = "127.0.0.1:2345"
...
```

Now we can run this configuration with `cargo embed` note that `cargo run` is configured to simply flash the program and does not start the GDB stub. Once you have run `cargo embed` you should see this line:

```sh
GDB stub listening at 127.0.0.1:2345
```

In another terminal connect to that stub using `gdb-multiarch`, passing in our executable so that gdb has a framework to supply useful information from.

```sh
gdb-multiarch target/thumbv6m-none-eabi/debug/B-minimal-blinky
```

Once gdb has started you will be presented with a command line interface with a `(gdb)` prompt. To connect to the stub and list the contents of the registers use these commands.

```sh
(gdb) target remote :2345
(gdb) info registers
```

Full use of gdb is out of the scope of this writing but the following are some useful commands for reference:
- `target remote :2345` - binds to gdb stub on localhost port 2345
- `info registers` or `i r` - prints contents of the CPU registers to the console
- `continue` or `c` - continue program execution until a signal is sent e.g. Ctrl+C or a breakpoint is hit
- `disassemble` - show disassembly of the current block that the PC points to
- `set print asm-demangle on` - clean up assembly output
- `break main.rs:41` - place a breakpoint on line 41 of `main.rs`

