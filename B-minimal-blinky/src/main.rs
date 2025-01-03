#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

use cortex_m_rt::entry;
use panic_halt as _;

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    // GPIO control
    const IO_BANK0_BASE: u32 = 0x4001_4000;
    const GPIO25_CTRL: *mut u32 = (0x0000_00CC + IO_BANK0_BASE) as *mut u32;
    // SIO control
    const SIO_BASE: u32 = 0xD000_0000;
    const GPIO_OE: *mut u32 = (0x0000_0020 + SIO_BASE) as *mut u32;
    const GPIO_OUT_XOR: *mut u32 = (0x0000_001C + SIO_BASE) as *mut u32;

    // Setting the GPIO 25 control register to be driven by the SIO module
    unsafe {
        let mut gpio25_ctrl: u32 = read_volatile(GPIO25_CTRL);
        gpio25_ctrl &= 0xCFFC_CCE0; // Clearing non-reserved
        gpio25_ctrl |= 0x0000_0005; // Setting function to F5 -> SIO
        write_volatile(GPIO25_CTRL, gpio25_ctrl);
    }

    // Enabling output on GPIO 25
    unsafe {
        let mut gpio_oe = read_volatile(GPIO_OE);
        gpio_oe |= 0x1 << 25;
        write_volatile(GPIO_OE, gpio_oe);
    }

    loop {
        // Toggle output level of GPIO 25
        unsafe {
            write_volatile(GPIO_OUT_XOR, 0x1 << 25);
        }
        delay_s(1);
    }
}

#[inline(always)]
fn delay_s(s: u32) {
    const EXTERNAL_XTAL_BASE_FREQ: u32 = 12_000_000;
    let cycles = s * EXTERNAL_XTAL_BASE_FREQ;
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}
