# Changelog

All notable changes to **drooling** are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) with Cargo's
pre-1.0 convention: a `0.x → 0.(x+1)` minor bump may break, announced
under `### Breaking`.

## [Unreleased]

## [0.2.0] — 2026-08-12

### Breaking

- No default feature anymore: consumers must enable exactly one of the
  `rp2040` / `rp2350` features. A plain `drooling = "0.1"` dependency no
  longer compiles for firmware targets; write
  `drooling = { version = "0.2", features = ["rp2040"] }` (or `rp2350`).
  Host builds without a chip feature remain valid.

### Changed

- The RP2040 demo is renamed from `examples/demo.rs` to
  `examples/demo_rp2040.rs`, mirroring `demo_rp2350.rs`
  (`cargo run --release --example demo_rp2040`).

### Added

- RP2350 support behind the opt-in `rp2350` feature, using the RP2350 boot
  ROM's reboot API. `rp2040` stays the default feature, so existing RP2040
  consumers are unchanged. The features are mutually exclusive.
- `examples/demo_rp2350.rs` (thumbv8m.main-none-eabihf, VID:PID 2e8a:0009),
  verified on a Waveshare RP2350-GEEK: enumeration with automatic WinUSB
  binding, button-free reflash, and — RP2350 only — single-shot
  `picotool load -f` on Windows.
- Per-chip linker memory layouts under `memory/`, selected together with
  the chip-specific rustflags by cfg() predicates in `.cargo/config.toml`
  (a config section named after the dotted thumbv8m triple is silently
  ignored by cargo).

## [0.1.2] — 2026-08-11

### Fixed

- The BOOTSEL request handler passed the raw GPIO pin number from
  `wValue` bits 9-14 to the boot ROM's `reset_to_usb_boot`, whose first
  argument is a pin **mask**. Harmless in the normal picotool flow (the
  activity-pin bit is never set), wrong whenever a host requests an
  activity LED.

### Changed

- `rp2040-hal` accepts `>=0.10, <0.13` so consumers on newer HALs
  resolve a single copy instead of carrying two.
- `rust-toolchain.toml` installs the `thumbv6m-none-eabi` target
  automatically.

### Added

- `protocol` module: host-testable parsing of the reset-request wire
  format.
- Platform notes in the README: the Linux/macOS single-shot runner
  line, udev rules, the 10 ms poll requirement, and VID/PID guidance
  for products.
- Verified board: Waveshare RP2040-ETH (W25Q32JV flash), alongside the
  Seeed XIAO RP2040.

## [0.1.1] — 2026-07-25

### Changed

- Renamed `vendor_reset_winusb::VendorResetWinUsb` to
  `drooling::PicotoolReset`. The class works on every host OS — the
  old name read as Windows-only. The old path remains as a deprecated
  alias.
- The release workflow guards tags against a `Cargo.toml` version
  mismatch and ships the demo firmware as UF2 and ELF release assets.

## [0.1.0] — 2026-07-25

Initial release: the Pico SDK compatible picotool reset interface as a
`usb-device` class, with BOS / Microsoft OS 2.0 descriptors for
automatic WinUSB binding on Windows. Includes a CDC-serial + reset
composite demo, host-side structural tests for the descriptor byte
arrays, and a button-free Windows flash flow (`tools/flash.cmd`).
