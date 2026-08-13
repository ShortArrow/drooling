# Contributing to drooling

## Development setup

Requirements: Rust (stable), `flip-link`, `picotool` v2.x in PATH,
and an RP2040 or RP2350 board.

```console
$ cargo install flip-link
$ cargo rp2040   # flashes the demo (see README for the first-flash step)
```

`rust-toolchain.toml` makes rustup install the `thumbv6m-none-eabi`
and `thumbv8m.main-none-eabihf` targets automatically. For RP2350, run
`cargo rp2350` instead.

## Architecture

`no_std` library crate plus a demo binary in `examples/`.

```
consumer firmware ──▶ picotool_reset (UsbClass) ──▶ ms_os_20 (descriptor data)
                                   └──▶ rp2040_hal::rom_data (BOOTSEL reboot)

drool (tools/drool) ──▶ nusb (reset) / picoboot (BOOTSEL flashing)
```

- **ms_os_20**: dependency-free descriptor byte arrays (BOS platform
  capability, MS OS 2.0 descriptor set). Compiles on any host, which is
  what makes the structural tests possible.
- **picotool_reset**: the `UsbClass` implementation — interface
  descriptor, BOS/MS OS 2.0 request handling, reset request handling.
  Target-gated (`arm` + `none`).

Design decisions are recorded in [adr/](adr/).

## Tests

```console
$ cargo test --lib --target x86_64-pc-windows-msvc    # host (Windows)
$ cargo test --lib --target x86_64-unknown-linux-gnu  # host (Linux)
$ cargo test -p drool                                 # host tool (8 tests)
```

Naming the host target explicitly keeps these tests host-only even if a
default build target is ever added back to `.cargo/config.toml`.

Descriptor changes must keep the structural tests green; they replicate
the Windows MS OS 2.0 validator rules (lengths, offsets, containment).

The `drool` tests cover the pure ELF-to-write-plan module — pages
synthesized by rounding segment starts down to 256-byte boundaries, gaps
filled with `0xFF`, consecutive pages coalesced, overlapping segments
rejected — plus one regression test pinning drool's need for a tokio
runtime context.

### Hardware test manifest

Everything below needs a physical RP2040 or RP2350 board on the bench,
which is what keeps these cases out of CI. On-target descriptor
verification also belongs here: flash the demo and confirm Windows
enumerates the device error-free (serial-number instance ID, COM port,
"Reset" interface bound to WinUSB), and that `picotool reboot -f -u`
works.

| Case | Command | Expected |
| ---- | ------- | -------- |
| Running device, full cycle | `cargo rp2040` / `cargo rp2350` | Reset, BOOTSEL, write, verify, ending in "Rebooting into the application"; the device re-enumerates as the demo |
| Device already in BOOTSEL | `drool run <ELF>` | Reset step skipped, flashes and restarts |
| Reset only | `drool reboot --app` | Device re-enumerates without being reflashed |

## Bilingual documentation

`X.md` / `X.jp.md` pairs (README, ROADMAP) must change together — CI
enforces this. ADRs are Japanese-canonical; English editions are optional
and indexed in `adr/README.md`.

## Release

1. Bump the version in `Cargo.toml`, move the `[Unreleased]` entries of
   `CHANGELOG.md` under the new `[X.Y.Z]` heading, and push to `main`.
2. Push a `v*` tag matching the version. CI gates the release on tests,
   builds the demo UF2/ELF, creates a GitHub release with generated
   notes, and publishes to crates.io via OIDC trusted publishing. Put
   `[skip publish]` in the tagged commit message to skip crates.io.

To release `drool`, bump the version in `tools/drool/Cargo.toml` before
tagging: the same workflow publishes it whenever that version is not on
crates.io yet.

`CHANGELOG.md` is English-only by convention and therefore exempt from
the bilingual tandem check.
