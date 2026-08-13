//! Pico SDK compatible vendor reset interface with Windows support
//!
//! Implements vendor-specific reset interface with BOS and MS OS 2.0 descriptors
//! for automatic WinUSB driver loading on Windows.

use usb_device::class_prelude::*;
use usb_device::control;
use usb_device::descriptor::BosWriter;
use usb_device::Result;

use crate::ms_os_20;

/// Interface class for vendor-specific interface
const TUSB_CLASS_VENDOR_SPECIFIC: u8 = 0xFF;

/// Reset interface subclass (must be 0x00)
const RESET_INTERFACE_SUBCLASS: u8 = 0x00;

/// Reset interface protocol (must be 0x01)
const RESET_INTERFACE_PROTOCOL: u8 = 0x01;

/// Vendor request: Reset to BOOTSEL mode
const RESET_REQUEST_BOOTSEL: u8 = 0x01;

/// Vendor request: Reset to flash (normal boot)
const RESET_REQUEST_FLASH: u8 = 0x02;

/// Pico SDK compatible vendor reset interface with Windows support
pub struct PicotoolReset {
    iface: InterfaceNumber,
    str_idx: StringIndex,
}

impl PicotoolReset {
    /// Create a new vendor reset interface with Windows support
    pub fn new<B: UsbBus>(alloc: &UsbBusAllocator<B>) -> PicotoolReset {
        PicotoolReset {
            iface: alloc.interface(),
            str_idx: alloc.string(),
        }
    }
}

impl<B: UsbBus> UsbClass<B> for PicotoolReset {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface_alt(
            self.iface,
            0, // alternate setting
            TUSB_CLASS_VENDOR_SPECIFIC,
            RESET_INTERFACE_SUBCLASS,
            RESET_INTERFACE_PROTOCOL,
            Some(self.str_idx),
        )?;

        Ok(())
    }

    fn get_string(&self, index: StringIndex, _lang_id: LangID) -> Option<&str> {
        if index == self.str_idx {
            Some("Reset")
        } else {
            None
        }
    }

    fn get_bos_descriptors(&self, writer: &mut BosWriter) -> Result<()> {
        // Write MS OS 2.0 Platform Capability Descriptor
        // writer.capability() adds 3 bytes: bLength, bDescriptorType, bDevCapabilityType
        // We need to provide: bReserved (1) + UUID (16) + platform data (7) = 24 bytes
        // Total: 3 (auto) + 24 (provided) = 27 bytes
        //
        // BOS_DESCRIPTOR structure:
        // [0..5]: BOS header (skip)
        // [5]: bLength (28) - skip
        // [6]: bDescriptorType (0x10) - skip
        // [7]: bDevCapabilityType (0x05) - skip
        // [8]: bReserved (0x00) - INCLUDE
        // [9..25]: UUID (16 bytes) - INCLUDE
        // [25..32]: Platform data (7 bytes) - INCLUDE

        writer.capability(
            0x05, // bDevCapabilityType: Platform
            &ms_os_20::BOS_DESCRIPTOR[8..], // bReserved + UUID + platform data (24 bytes)
        )?;
        Ok(())
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();

        // Windows fetches the MS OS 2.0 descriptor set with this vendor request.
        if req.request_type == control::RequestType::Vendor
            && req.recipient == control::Recipient::Device
            && req.request == ms_os_20::MS_OS_20_VENDOR_CODE
            && req.index == 7
        {
            let interface_num = u8::from(self.iface);
            let desc = ms_os_20::update_interface_number(interface_num);
            xfer.accept_with(&desc).ok();
            return;
        }

        if req.request_type != control::RequestType::Vendor
            || req.recipient != control::Recipient::Interface
            || req.index != u8::from(self.iface) as u16
        {
            return;
        }

        xfer.reject().ok();
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();

        // picotool sends Class requests to Interface, not Vendor requests
        // LIBUSB_REQUEST_TYPE_CLASS | LIBUSB_RECIPIENT_INTERFACE = 0x21
        if req.request_type != control::RequestType::Class
            || req.recipient != control::Recipient::Interface
            || req.index != u8::from(self.iface) as u16
        {
            return;
        }

        match req.request {
            RESET_REQUEST_BOOTSEL => {
                let (gpio_activity, disable_interface_mask) =
                    crate::protocol::bootsel_reset_args(req.value);

                if xfer.accept().is_err() {
                    return;
                }

                // Small delay to ensure USB transfer completes
                cortex_m::asm::delay(1000000); // ~8ms at 125MHz

                enter_bootsel(gpio_activity, disable_interface_mask)
            }

            RESET_REQUEST_FLASH => {
                if xfer.accept().is_err() {
                    return;
                }

                // Small delay to ensure USB transfer completes
                cortex_m::asm::delay(1000000); // ~8ms at 125MHz

                reboot_to_application()
            }

            _ => {
                xfer.reject().ok();
            }
        }
    }
}

/// Enter BOOTSEL mode. Never returns.
#[cfg(feature = "rp2040")]
fn enter_bootsel(gpio_activity_pin_mask: u32, disable_interface_mask: u32) -> ! {
    rp2040_hal::rom_data::reset_to_usb_boot(gpio_activity_pin_mask, disable_interface_mask);

    // rp2040-hal 0.10 types this as `()`, 0.12 as `!`; the loop makes both compile.
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Enter BOOTSEL mode. Never returns.
///
/// RP2350 has no GPIO activity pin parameter, so that mask is ignored.
#[cfg(feature = "rp2350")]
fn enter_bootsel(_gpio_activity_pin_mask: u32, disable_interface_mask: u32) -> ! {
    rp235x_hal::reboot::reboot(
        rp235x_hal::reboot::RebootKind::BootSel {
            msd_disabled: disable_interface_mask & 0x01 != 0,
            picoboot_disabled: disable_interface_mask & 0x02 != 0,
        },
        rp235x_hal::reboot::RebootArch::Normal,
    )
}

/// Reboot into the flashed application. Never returns.
#[cfg(feature = "rp2040")]
fn reboot_to_application() -> ! {
    unsafe {
        const WATCHDOG_BASE: u32 = 0x40058000;
        const CTRL_OFFSET: u32 = 0x00;
        const CTRL_TRIGGER_BIT: u32 = 1 << 31;

        let ctrl_reg = (WATCHDOG_BASE + CTRL_OFFSET) as *mut u32;
        ctrl_reg.write_volatile(CTRL_TRIGGER_BIT);
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Reboot into the flashed application. Never returns.
#[cfg(feature = "rp2350")]
fn reboot_to_application() -> ! {
    rp235x_hal::reboot::reboot(
        rp235x_hal::reboot::RebootKind::Normal,
        rp235x_hal::reboot::RebootArch::Normal,
    )
}
