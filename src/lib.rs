//! Pico SDK compatible picotool reset interface for RP2040 Rust firmware.
//!
//! Add [`vendor_reset_winusb::VendorResetWinUsb`] to a `usb-device` composite
//! device and `picotool reboot -f -u` (and on Linux/macOS `picotool load -f`)
//! can reboot the running firmware into BOOTSEL mode — no BOOTSEL button.
//! Windows binds WinUSB to the interface automatically via the bundled
//! BOS / Microsoft OS 2.0 descriptors ([`ms_os_20`]).
//!
//! The `UsbDeviceBuilder` on the caller's side must use, or enumeration
//! fails on Windows:
//!
//! ```ignore
//! UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
//!     .strings(&[StringDescriptors::new(LangID::EN_US) /* not EN */ ...])
//!     .unwrap()
//!     .usb_rev(UsbRev::Usb210)   // Windows requests BOS only from >= 0x0210
//!     .max_packet_size_0(64)     // rp2040-hal enumeration hazard below 18
//!     .unwrap()
//!     .composite_with_iads()
//!     .build();
//! ```
//!
//! See `examples/demo.rs` for a complete composite device (CDC serial +
//! reset interface + LED).

#![cfg_attr(not(test), no_std)]

pub mod ms_os_20;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod vendor_reset_winusb;
