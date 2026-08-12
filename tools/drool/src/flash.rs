//! Flashing a device that is already in BOOTSEL mode, over PICOBOOT.
//!
//! The `picoboot` crate is async, and `nusb`'s default features route its
//! `MaybeFuture` awaits through `tokio::task::spawn_blocking`, which panics
//! outside a Tokio runtime context. A current-thread runtime therefore drives
//! every PICOBOOT call here. The reset path in [`crate::reset`] needs none of
//! this: it uses `nusb`'s blocking `.wait()` API directly.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use picoboot::{Access, Connection, Picoboot};
use tokio::runtime::{Builder, Runtime};

use crate::plan::{write_plan, WritePlan, PAGE_SIZE};

/// Delay the device waits before rebooting, giving the reboot command time to
/// be acknowledged before the USB link drops.
const REBOOT_DELAY: Duration = Duration::from_millis(500);

/// Builds the runtime that drives the PICOBOOT calls.
///
/// `spawn_blocking` only needs the `rt` feature's blocking pool, so the
/// current-thread builder is left without the I/O and time drivers that
/// `enable_all` would add.
fn runtime() -> Result<Runtime> {
    Builder::new_current_thread()
        .build()
        .context("failed to start the Tokio runtime driving the PICOBOOT transfers")
}

/// Whether a PICOBOOT device is currently attached.
pub fn picoboot_device_present() -> Result<bool> {
    let rt = runtime()?;
    let devices = rt
        .block_on(Picoboot::list_devices(None))
        .context("failed to enumerate PICOBOOT devices")?;
    Ok(!devices.is_empty())
}

/// Waits for a PICOBOOT device to appear, polling every `interval`.
pub fn wait_for_picoboot(timeout: Duration, interval: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if picoboot_device_present()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "no PICOBOOT device appeared within {:.0?} of the BOOTSEL reboot request",
                timeout
            );
        }
        std::thread::sleep(interval);
    }
}

/// Reads `path`, plans the writes, and applies them to the attached BOOTSEL device.
pub fn flash_elf(path: &Path, run_after: bool) -> Result<()> {
    let bytes =
        std::fs::read(path).with_context(|| format!("failed to read ELF file {}", path.display()))?;
    let plan = write_plan(&bytes)
        .with_context(|| format!("failed to build a flash plan for {}", path.display()))?;

    let rt = runtime()?;
    rt.block_on(apply_plan(&plan, run_after))
}

/// Runs the whole PICOBOOT session on one connection.
///
/// Erase, write, verify and reboot share a single [`Connection`], whose
/// exclusive access and post-XIP state must persist across all of them.
async fn apply_plan(plan: &WritePlan, run_after: bool) -> Result<()> {
    let mut picoboot = Picoboot::from_first(None)
        .await
        .context("no RP2040/RP2350 device found in BOOTSEL mode")?;

    let conn = picoboot
        .connect()
        .await
        .context("failed to connect to the PICOBOOT device")?;
    println!("Connected to {} in BOOTSEL mode", conn.target());

    conn.set_exclusive_access(Access::ExclusiveAndEject)
        .await
        .context("failed to claim exclusive PICOBOOT access")?;

    // Required on RP2040 before flash access; harmless on RP2350.
    conn.exit_xip()
        .await
        .context("failed to exit XIP mode before flashing")?;

    for (addr, len) in &plan.erase {
        println!("Erasing {len:#x} bytes at {addr:#010x}...");
        conn.flash_erase(*addr, *len)
            .await
            .with_context(|| format!("failed to erase {len:#x} bytes at {addr:#010x}"))?;
    }

    let mut written = 0usize;
    for (addr, data) in &plan.write {
        conn.flash_write(*addr, data)
            .await
            .with_context(|| format!("failed to write {} bytes at {:#010x}", data.len(), addr))?;
        written += data.len();
        println!("Wrote {} bytes at {:#010x}", data.len(), addr);
    }
    println!("Wrote {written} bytes in {} chunk(s)", plan.write.len());

    verify_first_page(conn, plan).await?;

    if run_after {
        conn.reboot(REBOOT_DELAY)
            .await
            .context("failed to reboot the device into the application")?;
        println!("Rebooting into the application");
    } else {
        println!("Leaving the device in BOOTSEL mode (--no-run)");
    }

    Ok(())
}

/// Reads back the first page of the first write and compares it.
///
/// A spot check, not a full verify: it catches a device that acknowledged the
/// writes without committing them, at the cost of one extra transfer.
async fn verify_first_page(conn: &mut Connection, plan: &WritePlan) -> Result<()> {
    let Some((addr, data)) = plan.write.first() else {
        return Ok(());
    };
    let len = data.len().min(PAGE_SIZE as usize);

    let read_back = conn
        .flash_read(*addr, len as u32)
        .await
        .with_context(|| format!("failed to read back {len} bytes at {addr:#010x}"))?;

    if read_back != data[..len] {
        bail!(
            "verification failed: the {len} bytes read back from {addr:#010x} \
             differ from what was written"
        );
    }
    println!("Verified {len} bytes at {addr:#010x}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for a panic that only ever showed up on hardware.
    ///
    /// `nusb`'s default features route its `MaybeFuture` awaits through
    /// `tokio::task::spawn_blocking`, which panics with "there is no reactor
    /// running" unless a Tokio runtime context is active. Driving a real
    /// PICOBOOT call through [`runtime`] proves the builder used by the flash
    /// path supplies that context.
    ///
    /// No device is needed: enumeration reaches `spawn_blocking` whether or
    /// not anything is plugged in, and an empty list is a success.
    #[test]
    fn picoboot_calls_resolve_under_the_flash_runtime() {
        let rt = runtime().expect("the flash runtime builds");

        let devices = rt
            .block_on(Picoboot::list_devices(None))
            .expect("enumeration succeeds even with no device attached");

        println!("enumerated {} PICOBOOT device(s)", devices.len());
    }
}
