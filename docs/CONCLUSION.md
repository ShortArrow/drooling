# Investigation Record: picotool button-free flashing with Rust on RP2040

## 🟢 Result: WORKS (verified 2026-07-25)

Button-free reflashing of Rust firmware on RP2040 via picotool's vendor reset
interface **works on Windows**, implemented in pure Rust (no TinyUSB FFI).

**Verified with**:
- picotool v2.2.0-a4 (Windows)
- usb-device v0.3.2 / usbd-serial v0.2.2 / rp-pico v0.9
- Seeed XIAO RP2040, Windows 11 Pro (26200)

Verified cycle, twice in a row and then via `cargo run --release` alone:

```
flash → app boots → picotool reboot -f -u (no button) → BOOTSEL → flash → app boots
```

## Correction of the 2025-11-01 conclusion

An earlier version of this document concluded that picotool integration was
"NOT practically achievable" in Rust because "the usb-device crate does not
support BOS descriptors or MS OS 2.0 descriptors". Both claims were wrong:

- usb-device 0.3 has `UsbClass::get_bos_descriptors`, and this repository
  already contained a working BOS + MS OS 2.0 implementation
  (`src/vendor_reset_winusb.rs`, `src/ms_os_20.rs`).
- The observed failure (`CM_PROB_FAILED_START`) was not a fundamental
  ecosystem limitation. It was a stack of five independent, fixable problems,
  listed below. The November tests were additionally contaminated by problem 1,
  which made every firmware variant fail identically and mimicked a
  "fundamental" incompatibility.

## Root causes found (all five were required)

1. **Stale Zadig/libwdi driver on the host.** A libwdi-generated INF
   (targeting exactly `USB\VID_2E8A&PID_000A`, device-level) was force-binding
   WinUSB to the whole composite device, preventing `usbccgp` from splitting
   interfaces. Every firmware variant failed with `CM_PROB_FAILED_START`
   regardless of its descriptors. Removed with `pnputil /delete-driver`
   plus the stale devnode and the `usbflags` registry cache entry.

2. **EP0 max packet size left at the usb-device default of 8 bytes.**
   rp2040-hal has a documented Windows enumeration hazard when EP0 max packet
   size is below 18 bytes. Fixed with `.max_packet_size_0(64)`.

3. **MS OS 2.0 descriptor set: `bFirstInterface` patched at the wrong
   offset.** The patch wrote to offset 26 (the Compatible ID descriptor's
   `wLength` field, corrupting 20 → 2) instead of offset 22. Windows rejected
   the set ("Validation Failure of MS OS 2.0 Descriptor Set" in the USBHUB3
   ETW trace) and, because the device declares bcdUSB 0x0210, failed the whole
   enumeration.

4. **MS OS 2.0 subset headers: lengths excluded their own header size.**
   `wTotalLength` of the configuration subset and `wSubsetLength` of the
   function subset must include the 8-byte header itself
   (174 = 10 + [8 + [8 + 20 + 128]]). Both were 8 short, making the registry
   property descriptor overflow its parent subset. Same ETW validation
   failure, reported at offset 165.

5. **Language ID mismatch: `LangID::EN` (0x0009) instead of `EN_US`
   (0x0409).** usb-device rejects (stalls) string requests whose language ID
   does not exactly match a configured `StringDescriptors` set. Windows
   requests strings with 0x0409, so manufacturer/product/serial reads all
   stalled ("Request for Serial Number String Descriptor Failed",
   STATUS_INFO_LENGTH_MISMATCH). With bcdUSB 0x0200 Windows tolerates this
   (device enumerates, instance ID falls back to the port path); with
   bcdUSB 0x0210 it fails enumeration.

## Diagnostic techniques that located the bugs

- Windows PnP state: `Get-PnpDevice` / `Get-PnpDeviceProperty`
  (`DEVPKEY_Device_ProblemStatus` gave NTSTATUS codes;
  `DEVPKEY_Device_Service` + `DriverInfPath` exposed the libwdi hijack).
- Controlled single-variable bisect: a bcdUSB 0x0200 build enumerated fine,
  isolating the failure to the BOS / MS OS 2.0 path.
- USBHUB3 ETW tracing (`logman start -ets -p Microsoft-Windows-USB-USBHUB3`,
  then `Get-WinEvent -Path trace.etl`): named the exact failing stage —
  descriptor-set validation, then the serial-number string request — without
  any USB hardware sniffer.
- Hub IOCTL (`IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX`) confirmed
  `DeviceFailedEnumeration` at the hub level.

## Remaining platform limitation (host side, not firmware)

picotool itself rejects single-shot forced commands for RP2040 on Windows:

```
ERROR: Forced commands do not work with RP2040 on Windows -
you can force reboot into BOOTSEL mode via 'picotool reboot -f -u' instead.
```

The supported Windows flow is two-step — `picotool reboot -f -u`, then
`picotool load` — which `flash.cmd` (the cargo runner) automates. On Linux and
macOS the same firmware should work with single-shot `picotool load -f`
(not yet verified here).

## Regression protection

`cargo test --lib --target x86_64-pc-windows-msvc` runs host-side structural
tests over the MS OS 2.0 descriptor set and BOS descriptor
(`src/ms_os_20.rs`), covering the length/offset bug class of problems 3 and 4.
