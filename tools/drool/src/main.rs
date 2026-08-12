//! `drool` — button-free flasher for firmware built on the `drooling` reset
//! interface.
//!
//! A running device is rebooted into BOOTSEL over its vendor reset interface,
//! then flashed over PICOBOOT, so the BOOTSEL button is only ever needed for
//! the very first flash.

mod flash;
mod plan;
mod reset;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

use reset::RebootTarget;

/// How long `run` waits for the ROM to re-enumerate as PICOBOOT.
const BOOTSEL_TIMEOUT: Duration = Duration::from_secs(10);
const BOOTSEL_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Parser)]
#[command(
    name = "drool",
    about = "Button-free flasher for RP2040/RP2350: reboots running firmware via the Pico reset interface and flashes ELFs over PICOBOOT"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Reboot a running device into BOOTSEL (or back into the application).
    Reboot {
        /// Reboot into the application instead of BOOTSEL.
        #[arg(long)]
        app: bool,
    },
    /// Flash an ELF to a device that is already in BOOTSEL mode.
    Flash {
        /// The ELF to flash.
        elf: PathBuf,
        /// Leave the device in BOOTSEL mode instead of running the firmware.
        #[arg(long)]
        no_run: bool,
    },
    /// Reboot into BOOTSEL if needed, flash the ELF, then run it.
    ///
    /// This is the verb intended for a cargo runner, which appends the ELF path.
    Run {
        /// The ELF to flash.
        elf: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Reboot { app } => cmd_reboot(app),
        Command::Flash { elf, no_run } => flash::flash_elf(&elf, !no_run),
        Command::Run { elf } => cmd_run(&elf),
    }
}

fn cmd_reboot(app: bool) -> Result<()> {
    let direction = if app {
        RebootTarget::Application
    } else {
        RebootTarget::Bootsel
    };

    let Some(target) = reset::find_reset_device()? else {
        bail!(
            "no running device exposes the drooling reset interface \
             (vendor class 0xFF, subclass 0x00, protocol 0x01)"
        );
    };

    println!("Found reset interface on {}", target.describe());
    reset::reboot(&target, direction)?;
    println!("Rebooting into {}", direction.describe());
    Ok(())
}

fn cmd_run(elf: &std::path::Path) -> Result<()> {
    match reset::find_reset_device()? {
        Some(target) => {
            println!("Found reset interface on {}", target.describe());
            reset::reboot(&target, RebootTarget::Bootsel)?;
            println!("Rebooting into BOOTSEL, waiting for the ROM to enumerate...");
            flash::wait_for_picoboot(BOOTSEL_TIMEOUT, BOOTSEL_POLL_INTERVAL)?;
        }
        None if flash::picoboot_device_present()? => {
            println!("Device is already in BOOTSEL mode");
        }
        None => bail!(
            "found neither a running device with the drooling reset interface \
             (vendor class 0xFF, subclass 0x00, protocol 0x01) nor a device in BOOTSEL mode"
        ),
    }

    flash::flash_elf(elf, true)
}
