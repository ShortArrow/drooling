# Roadmap

[English](ROADMAP.md) | [日本語](ROADMAP.jp.md)

Grouped by area, not by priority. Settled designs live in [adr/](adr/).

## Cross-platform verification

The firmware side is platform-independent, but only Windows has been
verified end to end.

- Verify `drool` end to end on Linux and macOS
- Document the udev rules Linux needs for the reset interface and the
  BOOTSEL device

## RISC-V flavor of RP2350

- Build and verify the demo for `riscv32imac-unknown-none-elf`, the
  RP2350's second architecture; the ARM (thumbv8m) flavor is done

## Demo modernization

- Replace the `static mut` USB bus allocator pattern in `examples/demo_rp2040.rs`
  with a warning-free idiom
- Consider an `embassy-usb` variant of the reset interface (embassy has
  first-class MS OS 2.0 support in its `msos` module)

## Publish drool

- Wire `drool` (on crates.io since 0.1.0) into Trusted Publishing and
  the release workflow; prebuilt binaries can
  follow

## Repository hygiene

- Retire `variants/` once the pure-Rust path has soaked
