//! Pure-Rust picotool `-f` support on RP2350 (Pico 2).
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

use rp235x_hal as hal;

use panic_halt as _;

use usb_device::{class_prelude::UsbBusAllocator, device::UsbRev, prelude::*, LangID};
use usbd_serial::SerialPort;

use drooling::PicotoolReset;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
    unsafe {
        USB_BUS = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            pac.USB,
            pac.USB_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )));
    }
    let usb_bus = unsafe { USB_BUS.as_ref().unwrap() };

    let mut serial = SerialPort::new(usb_bus);
    let mut picotool = PicotoolReset::new(usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2e8a, 0x0009))
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("Raspberry Pi")
            .product("Pico 2")
            .serial_number("123456")])
        .unwrap()
        .usb_rev(UsbRev::Usb210)
        .max_packet_size_0(64)
        .unwrap()
        .composite_with_iads()
        .build();

    loop {
        usb_dev.poll(&mut [&mut serial, &mut picotool]);
    }
}
