#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use rp2040_boot2;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionSio, Pin, PullDown, SioOutput},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[entry]
fn main() -> ! {
    // Get peripherals
    let mut pac = pac::Peripherals::take().unwrap();

    // Initialize watchdog and clocks
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let _clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up LED (GPIO 25 on Pico)
    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio25.reconfigure();

    // Blink LED
    loop {
        led.set_high().ok();
        cortex_m::asm::delay(10_000_000); // ~80ms at 125MHz
        led.set_low().ok();
        cortex_m::asm::delay(10_000_000);
    }
}
