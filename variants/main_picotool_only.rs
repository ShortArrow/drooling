#![no_std]
#![no_main]

use rp_pico as bsp;
use bsp::hal;
use hal::pac;

use cortex_m_rt::entry;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use panic_halt as _;

use usb_device::{prelude::*, class_prelude::UsbBusAllocator, LangID};
use usbd_picotool_reset::{PicoToolReset, DefaultConfig};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
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

    // USB bus
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

    // ONLY PicoToolReset (no SerialPort)
    let mut picotool: PicoToolReset<_, DefaultConfig> = PicoToolReset::new(usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .strings(&[StringDescriptors::new(LangID::EN)
            .manufacturer("Raspberry Pi")
            .product("Pico Test")
            .serial_number("TEST123")])
        .unwrap()
        .device_class(0xFF) // Vendor specific
        .build();

    // LED
    let sio = hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.led.into_push_pull_output();

    let mut counter = 0u32;

    loop {
        usb_dev.poll(&mut [&mut picotool]);

        counter = counter.wrapping_add(1);
        if counter % 500_000 == 0 {
            led.toggle().ok();
        }
    }
}
