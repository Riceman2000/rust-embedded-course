#![no_std]
#![no_main]

// Allocated types
extern crate alloc;
use alloc::vec::Vec;

use bsp::entry;
use defmt::info;
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionSio, Pin, PinId, PullType, SioOutput},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[global_allocator]
static HEAP: Heap = Heap::empty();

// External high-speed crystal on the pico board is 12Mhz
const EXTERNAL_XTAL_BASE_FREQ: u32 = 12_000_000;

#[derive(Clone, Copy, Debug)]
enum Action {
    High,
    Low,
}

#[entry]
fn main() -> ! {
    info!("Program start");

    // Heap initialization
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    // Peripheral initialization
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // Initialize clocks with the known external frequency
    let clocks = init_clocks_and_plls(
        EXTERNAL_XTAL_BASE_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    // Demonstrate heap allocations
    let mut action_tape = Vec::new();
    action_tape.push(Action::High);
    action_tape.push(Action::Low);
    action_tape.push(Action::High);
    action_tape.push(Action::Low);
    action_tape.push(Action::Low);
    action_tape.push(Action::Low);

    loop {
        // Use of a Rust iterator
        for &action in &action_tape {
            led_action(&mut led_pin, action);
            delay.delay_ms(250);
        }
    }
}

/// Set an LED based on the [`Action`] enum
/// Note that this function can only work with pins that have been properly configured to drive a
/// GPIO signal
fn led_action<I: PinId, P: PullType>(
    led_pin: &mut Pin<I, FunctionSio<SioOutput>, P>,
    action: Action,
) {
    match action {
        Action::High => {
            info!("On!");
            led_pin.set_high().unwrap();
        }
        Action::Low => {
            info!("Off!");
            led_pin.set_low().unwrap();
        }
    }
}
