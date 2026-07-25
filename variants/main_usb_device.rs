#![no_std]
#![no_main]

use rp_pico as bsp;
use bsp::hal;
use hal::pac;

use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use panic_halt as _;

use usb_device::{prelude::*, class_prelude::UsbBusAllocator, LangID, device::UsbRev};
use usbd_serial::SerialPort;
use usbd_picotool_reset::{PicoToolReset, DefaultConfig};

#[entry]
fn main() -> ! {
    // Get peripherals
    let mut pac = pac::Peripherals::take().unwrap();

    // Initialize watchdog and clocks
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up USB bus (MUST be static for proper lifetime)
    static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
    unsafe {
        USB_BUS = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )));
    }
    let usb_bus = unsafe { USB_BUS.as_ref().unwrap() };

    // Set up USB Serial and PicoTool Reset (testing original crate)
    let mut serial = SerialPort::new(usb_bus);
    let mut picotool: PicoToolReset<_, DefaultConfig> = PicoToolReset::new(usb_bus);

    // Create USB device with EXACT VID/PID required by picotool
    // CRITICAL: Use MISC class with IAD protocol for composite device
    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .strings(&[StringDescriptors::new(LangID::EN)
            .manufacturer("Raspberry Pi")
            .product("Pico")
            .serial_number("123456")])
        .unwrap()
        .composite_with_iads() // MISC class (0xEF) with IAD protocol
        .build();

    // Set up LED (GPIO 25 on Pico)
    let sio = hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.led.into_push_pull_output();

    let mut led_state = false;
    let mut counter = 0u32;

    // CRITICAL: Main loop must NEVER exit for picotool -f to work!
    loop {
        // Poll USB - MUST be called regularly
        usb_dev.poll(&mut [&mut serial, &mut picotool]);

        // Blink LED slowly (every ~500k loops)
        counter = counter.wrapping_add(1);
        if counter % 500_000 == 0 {
            led_state = !led_state;
            if led_state {
                led.set_high().ok();
            } else {
                led.set_low().ok();
            }
        }
    }
}
