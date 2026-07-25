//! Pure-Rust picotool `-f` support on RP2040.
//!
//! USB composite device: CDC serial + Pico SDK compatible vendor reset
//! interface with BOS / MS OS 2.0 descriptors for Windows WinUSB binding.
//!
//! Requirements for Windows to bind WinUSB automatically:
//! - bcdUSB 0x0210 (`UsbRev::Usb210`) so Windows requests the BOS descriptor
//! - `control-buffer-256` feature on usb-device (MS OS 2.0 set is 174 bytes)
//! - Misc/IAD device class (`composite_with_iads`)

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

use drooling::PicotoolReset;

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

    let mut serial = SerialPort::new(usb_bus);
    let mut picotool = PicotoolReset::new(usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2e8a, 0x000a))
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("Raspberry Pi")
            .product("Pico")
            .serial_number("123456")])
        .unwrap()
        .usb_rev(UsbRev::Usb210)
        .max_packet_size_0(64)
        .unwrap()
        .composite_with_iads()
        .build();

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

    loop {
        usb_dev.poll(&mut [&mut serial, &mut picotool]);

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
