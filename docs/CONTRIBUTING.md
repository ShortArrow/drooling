# Contributing to drooling

## Development setup

Requirements: Rust (stable) with the `thumbv6m-none-eabi` target,
`flip-link`, `picotool` v2.x in PATH, and an RP2040 board.

```console
$ cargo install flip-link
$ cargo run --release --example demo   # flashes the demo (see README for the first-flash step)
```

`rust-toolchain.toml` makes rustup install the `thumbv6m-none-eabi`
target automatically.

## Architecture

`no_std` library crate plus a demo binary in `examples/`.

```
consumer firmware ──▶ picotool_reset (UsbClass) ──▶ ms_os_20 (descriptor data)
                                   └──▶ rp2040_hal::rom_data (BOOTSEL reboot)
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
```

The explicit `--target` is required because `.cargo/config.toml` sets the
default build target to `thumbv6m-none-eabi`.

Descriptor changes must keep the structural tests green; they replicate
the Windows MS OS 2.0 validator rules (lengths, offsets, containment).
On-target verification: flash the demo and confirm Windows enumerates the
device error-free (serial-number instance ID, COM port, "Reset" interface
bound to WinUSB) and `picotool reboot -f -u` works.

## Bilingual documentation

`X.md` / `X.jp.md` pairs (README, ROADMAP) must change together — CI
enforces this. ADRs are Japanese-canonical; English editions are optional
and indexed in `adr/README.md`.

## Release

Push a `v*` tag. CI gates the release on tests, builds the demo UF2/ELF,
creates a GitHub release with generated notes, and publishes to crates.io
via OIDC trusted publishing. Put `[skip publish]` in the tagged commit
message to skip crates.io.
