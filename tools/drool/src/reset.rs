//! Talking to a *running* device through the drooling reset interface.

use std::time::Duration;

use anyhow::{Context, Result};
use nusb::transfer::{ControlOut, ControlType, Recipient};
use nusb::MaybeFuture;

/// The vendor interface drooling exposes: vendor class, reset subclass and
/// protocol, matching the Pico SDK's picotool reset interface.
const RESET_CLASS: u8 = 0xFF;
const RESET_SUBCLASS: u8 = 0x00;
const RESET_PROTOCOL: u8 = 0x01;

/// Vendor request that reboots the device into BOOTSEL.
const REQUEST_BOOTSEL: u8 = 0x01;
/// Vendor request that reboots the device back into the application.
const REQUEST_APP: u8 = 0x02;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(1);

/// Where the reset interface was found.
pub struct ResetTarget {
    pub vid: u16,
    pub pid: u16,
    pub interface: u8,
    info: nusb::DeviceInfo,
}

impl ResetTarget {
    pub fn describe(&self) -> String {
        format!(
            "{:04x}:{:04x} interface {}",
            self.vid, self.pid, self.interface
        )
    }
}

/// Finds the first connected device exposing the drooling reset interface.
///
/// Any VID/PID is accepted: the interface descriptor triple is the contract,
/// not the vendor identity.
pub fn find_reset_device() -> Result<Option<ResetTarget>> {
    let devices = nusb::list_devices()
        .wait()
        .context("failed to enumerate USB devices")?;

    for info in devices {
        let matching = info.interfaces().find(|itf| {
            itf.class() == RESET_CLASS
                && itf.subclass() == RESET_SUBCLASS
                && itf.protocol() == RESET_PROTOCOL
        });

        if let Some(itf) = matching {
            return Ok(Some(ResetTarget {
                vid: info.vendor_id(),
                pid: info.product_id(),
                interface: itf.interface_number(),
                info,
            }));
        }
    }

    Ok(None)
}

/// Which way to reboot a running device.
#[derive(Clone, Copy)]
pub enum RebootTarget {
    Bootsel,
    Application,
}

impl RebootTarget {
    fn request(self) -> u8 {
        match self {
            RebootTarget::Bootsel => REQUEST_BOOTSEL,
            RebootTarget::Application => REQUEST_APP,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            RebootTarget::Bootsel => "BOOTSEL",
            RebootTarget::Application => "the application",
        }
    }
}

/// Sends the reset interface's reboot request.
///
/// The interface must be claimed first: on Windows nusb routes an
/// interface-recipient control transfer through the claimed interface, and
/// requires the low byte of `index` to name that same interface.
pub fn reboot(target: &ResetTarget, direction: RebootTarget) -> Result<()> {
    let device = target
        .info
        .open()
        .wait()
        .with_context(|| format!("failed to open USB device {}", target.describe()))?;

    let interface = device
        .claim_interface(target.interface)
        .wait()
        .with_context(|| {
            format!(
                "failed to claim reset interface {} on {:04x}:{:04x}",
                target.interface, target.vid, target.pid
            )
        })?;

    interface
        .control_out(
            ControlOut {
                control_type: ControlType::Class,
                recipient: Recipient::Interface,
                request: direction.request(),
                value: 0,
                index: u16::from(target.interface),
                data: &[],
            },
            CONTROL_TIMEOUT,
        )
        .wait()
        .with_context(|| {
            format!(
                "reset request {:#04x} (reboot to {}) was rejected by {}",
                direction.request(),
                direction.describe(),
                target.describe()
            )
        })?;

    Ok(())
}
