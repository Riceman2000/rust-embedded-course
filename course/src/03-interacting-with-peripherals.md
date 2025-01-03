# Interacting With Peripherals

# Hardware Selection
Modern microcontrollers are selected based on a long list of factors, with each factor being weighted differently depending on the engineer and project. Some of the major considerations include:

## Available Peripherals
A unique piece of hardware internal to the microcontroller that can affect the developer experience, common examples include:
- UART - Universal Asynchronous Receiver / Transmitter
- SPI/QSPI - [Quad] Serial Peripheral Interface
- I2C - Inter-Integrated Circuit
- CAN Bus - Controller Area Network bus
- USB - Universal Serial Bus
- Clocks/timers

There are countless different peripherals available for microcontrollers, all with different niche uses. An engineer in the automotive space will likely prioritize the CAN Bus and USB to interface with a driver's smartphone.

## Processor Capability
What the processor on the microcontroller is able to do and what operating modes are available. Certain applications may require a multi-core microcontroller; others may demand a microcontroller that has deep sleep and/or low power modes. This category can also include general performance requirements. Some applications require significant data throughput from one peripheral to another, post-processing, or low latency.

## Price
Often not a huge concern to the hobbyist, but price can make or break a project's feasibility. Consider [Apple's sales figures](https://www.statista.com/statistics/263402/apples-iphone-revenue-since-3rd-quarter-2007/) as an example. A change in price of one component by $0.01 at 90,000,000 units a year equates to $900,000 off their bottom line, and $0.05 equates to a $4,500,000 loss. This level of price optimization has to be performed for every distinct component in the product along with the manufacturing methods.

## Development Ecosystem/Developer Experience
How easy is it to program the microcontroller? This includes factors such as library availability, documentation, and IDE support. Another major consideration is the developer's level of experience working with a specific platform or vendor, as learning a new architecture can be time-consuming.

# Working With GPIO
For this example project, our goal is to turn a built-in LED on and off. To accomplish this, we need to properly configure the GPIO and SIO modules of the Pi Pico. Using the board datasheet:

https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf

We can determine that the LED is wired into `GP25`. Using the controller datasheet section **1.4.3. GPIO Functions**, we can see what that pin is capable of. Pins can have multiple functions that the user can select; this selection process is referred to as multiplexing or muxing, after the digital component by the same name.

| GPIO | F1       | F2       | F3       | F4     | F5  | F6   | F7   | F8           | F9           |
|------|----------|----------|----------|--------|-----|------|------|--------------|--------------|
| 25   | SPI1 CSn | UART1 RX | I2C0 SCL | PWM4 B | SIO | PIO0 | PIO1 | CLOCK GPOUT3 | USB VBUS DET |

Of these different functions, a few could do what we want:
- `SIO`
- `PIO0` & `PIO1`
- `CLOCK GPOUT3`

The table below the function table lists the descriptions of each of the different peripherals available. Using the SIO module is going to be the easiest way to accomplish our LED blinking goal, even if it does not leverage the full ability of the processor.

## Selecting SIO
Of the functions listed in the table above, we need to select the `SIO` module to drive `GP25`. The column that `SIO` is in is `F5`, meaning we need to write a `5` to the function select field of the control register for `GP25`. To find the function select field, we need to look in section **2.19.6.1. IO - User Bank**. This section specifies that the `IO_BANK0_BASE` registers begin at an offset of `0x4001_4000`. The `GPIO0_CTRL` register is offset `0x0cc` from the base address. Representing that in code would be:

```rust
#[entry]
fn main() -> ! {
    // GPIO control
    const IO_BANK0_BASE: u32 = 0x4001_4000;
    const GPIO25_CTRL: *mut u32 = (0x0000_00CC + IO_BANK0_BASE) as *mut u32;
    ...
```

Configuring this register will require reading the current value, modifying it, then writing it again. The volatile read and write options are `unsafe` operations in Rust as they rely on the programmer to ensure the addresses are correct, the type specified matches the data at that address, and that the value once read is handled properly and not destroyed before its final use. The code to modify the `GPIO25_CTRL` register function field is:

```rust
    // Setting the GPIO 25 control register to be driven by the SIO module
    unsafe {
        let mut gpio25_ctrl: u32 = read_volatile(GPIO25_CTRL);
        gpio25_ctrl &= 0xCFFC_CCE0; // Clearing non-reserved
        gpio25_ctrl |= 0x0000_0005; // Setting function to F5 -> SIO
        write_volatile(GPIO25_CTRL, gpio25_ctrl);
    }
```

Note the use of the bitwise assignment operators `&=` and `|=` to clear the non-reserved values to their non-inverted states, then set the function select to 5 as described in the earlier table. The common terminology here comes from digital logic, to **set** means to change a value to `1`, and to **clear** is to change a value to `0`.

## Configuring SIO
The base address of the `SIO` registers is defined in section **2.3.1.7. List of Registers** as `0xd000_0000`, referred to as `SIO_BASE`. The layout of these registers is different from the `GPIOXX_CTRL` register modified earlier. Each register is a 32-bit value with each bit associated with a particular pin. To enable output on the selected `GP25` pin, we need the `GPIO_OE` register with an offset of `0x020`. Because we are concerned with `GP25`, we need the 25th bit in the register to be set. The following code enables output on the selected pin:

```rust 
    // SIO control
    const SIO_BASE: u32 = 0xD000_0000;
    const GPIO_OE: *mut u32 = (0x0000_0020 + SIO_BASE) as *mut u32;

    // Enabling output on GPIO 25
    unsafe {
        let mut gpio_oe = read_volatile(GPIO_OE);
        gpio_oe |= 0x1 << 25;
        write_volatile(GPIO_OE, gpio_oe);
    }
```

## Driving The Pin
Finally, we need to drive the output on `GP25` and we do that with the `OUT` registers. In this case, we will use the `XOR` variant because we are always toggling the state. This will save us from having to manage the state ourselves in software. The `GPIO_OUT_XOR` register has an offset of `0x01c` and we will modify it with this code:

```rust
    const GPIO_OUT_XOR: *mut u32 = (0x0000_001C + SIO_BASE) as *mut u32;

    loop {
        // Toggle output level of GPIO 25
        unsafe {
            write_volatile(GPIO_OUT_XOR, 0x1 << 25);
        }
    }
```

# Delay
If we upload this code as it is now, the LED will turn on but will appear to be lit to about half brightness. That is because the microcontroller is executing the instructions in that loop as fast as possible with no delay. To us, it appears the LED is just less bright, but it is actually flickering based on the clock speed of our microcontroller.

To avoid this, let's create a delay function. We can utilize the `cortex_m::asm::nop();` function, which will invoke the `nop` assembly instruction, wasting a clock cycle. But we need a way to determine how many of those assembly instructions we need, and we also want to utilize a loop instead of baking 12 million delay instructions into the binary. `Appendix B` of the [board datasheet](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) shows that the external clock included on the Pi Pico is 12MHz, meaning that for a 1 second delay we need to waste 12 million clock cycles. Here is an example of a simple delay function:

```rust
#[inline(always)]
fn delay_s(s: u32) {
    const EXTERNAL_XTAL_BASE_FREQ: u32 = 12_000_000;
    let cycles = s * EXTERNAL_XTAL_BASE_FREQ;
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}
```

# Conclusions
The code for the full blinking example can be found in the `B-minimal-blinky` directory. This chapter explored accessing memory-mapped registers to use peripherals. This direct approach is time-consuming and error-prone, both in programming and in research required poring over datasheets searching for addresses. The programmer must also be absolutely certain that all of the invariants are met for different hardware configurations. This is trivial at first, but as your program grows in scope, it quickly gets out of hand. Because of this, developers often use BSPs or Board Support Packages. These pieces of software wrap around the direct memory access in a way that prevents improper usage (to a degree) and allows for a more readable program. BSPs also have the added benefit of detaching the functionality you programmed into your project from the hardware it is running on, allowing for more portability. Later chapters will cover the BSP available for the Pi Pico and the levels of abstraction it is built upon.

The next chapter will be a brief look into RTT and GDB as tools that we can leverage to make embedded development easier.

