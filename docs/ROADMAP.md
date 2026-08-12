# Roadmap

[English](ROADMAP.md) | [日本語](ROADMAP.jp.md)

Grouped by area, not by priority. Settled designs live in [adr/](adr/).

## Cross-platform verification

The firmware side is platform-independent, but only Windows has been
verified end to end.

- Verify single-shot `picotool load -f` on Linux and macOS
- Document a runner setup per platform (the two-step `flash.cmd` flow is
  only needed on Windows)

## RISC-V flavor of RP2350

- Build and verify the demo for `riscv32imac-unknown-none-elf`, the
  RP2350's second architecture; the ARM (thumbv8m) flavor is done

## Demo modernization

- Replace the `static mut` USB bus allocator pattern in `examples/demo.rs`
  with a warning-free idiom
- Consider an `embassy-usb` variant of the reset interface (embassy has
  first-class MS OS 2.0 support in its `msos` module)

## All-Rust host tooling

- Replace the C++ picotool on the host side with a Rust tool: send the
  vendor reset request with `nusb` and speak the PICOBOOT protocol for
  loading, collapsing the flash flow into one Rust binary

## Repository hygiene

- Retire `variants/` once the pure-Rust path has soaked
