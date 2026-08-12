//! Pico SDK compatible picotool reset interface for RP2040 Rust firmware.
//!
//! Add [`PicotoolReset`] to a `usb-device` composite device and
//! `picotool reboot -f -u` (and on Linux/macOS `picotool load -f`)
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
//!     .max_packet_size_0(64)     // rp2040-hal enumeration hazard below 18 bytes
//!     .unwrap()
//!     .composite_with_iads()
//!     .build();
//! ```
//!
//! Call `usb_dev.poll(...)` at least once every 10 ms while connected —
//! preferably continuously from the main loop or a USB interrupt — or
//! enumeration and reset requests will be missed.
//!
//! See `examples/demo_rp2040.rs` for a complete composite device (CDC serial +
//! reset interface + LED) and `examples/demo_rp2350.rs` for the RP2350
//! flavor (feature `rp2350`, mutually exclusive with the default `rp2040`).

#![cfg_attr(not(test), no_std)]

#[cfg(all(feature = "rp2040", feature = "rp2350"))]
compile_error!("features `rp2040` and `rp2350` are mutually exclusive");

pub mod ms_os_20;
pub mod protocol;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod picotool_reset;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use picotool_reset::PicotoolReset;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[deprecated(since = "0.1.1", note = "renamed to `drooling::PicotoolReset`")]
pub mod vendor_reset_winusb {
    //! Renamed to [`crate::picotool_reset`].
    pub use crate::picotool_reset::PicotoolReset as VendorResetWinUsb;
}
