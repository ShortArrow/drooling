# drooling

[English](README.md) | [日本語](docs/README.jp.md)

Button-free flashing for Rust firmware on RP2040, using picotool's vendor
reset interface — pure Rust, Pico SDK compatible, works on Windows.

Add the `PicotoolReset` class to your USB composite device and
`picotool reboot -f -u` reboots the running firmware into BOOTSEL mode, no
BOOTSEL button. Windows binds WinUSB automatically via the bundled
BOS / Microsoft OS 2.0 descriptors — no Zadig, no manual driver setup.

The name: a USB Type-C port looks like the mouth of a slime about to
drool. What leaks out of this one, drool-like, is control — the ability
to reboot and reflash the board seeps out of the USB port with no button
pressed.

## Using from your project

```toml
# Cargo.toml
[dependencies]
# RP2040
drooling = { version = "0.2", features = ["rp2040"] }

# RP2350
drooling = { version = "0.2", features = ["rp2350"] }
```

The chip features are mutually exclusive and there is no default, so
exactly one of them must be chosen.

```rust
use drooling::PicotoolReset;

// after setting up the rp2040-hal USB bus:
let mut serial = SerialPort::new(&usb_bus);
let mut picotool = PicotoolReset::new(&usb_bus);

let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
    .strings(&[StringDescriptors::new(LangID::EN_US)  // EN_US, not EN
        .manufacturer("Raspberry Pi")
        .product("Pico")
        .serial_number("123456")])
    .unwrap()
    .usb_rev(UsbRev::Usb210)   // Windows requests BOS only from >= 0x0210
    .max_packet_size_0(64)     // rp2040-hal enumeration hazard below 18 bytes
    .unwrap()
    .composite_with_iads()
    .build();

loop {
    usb_dev.poll(&mut [&mut serial, &mut picotool]);
    // your application
}
```

All four builder settings marked above are load-bearing on Windows —
enumeration fails or WinUSB is not bound if any is missing. The required
`usb-device` feature `control-buffer-256` is enabled transitively by
depending on this crate.

For button-free `cargo run`, install the flasher and point your
`runner` at it:

```sh
cargo install drool
```

```toml
# .cargo/config.toml
runner = "drool run"
```

This repository runs its bundled copy as `cargo run -q -p drool -- run`.

Picotool alternatives and platform notes live in
[docs/FLASHING.md](docs/FLASHING.md).

## Requirements

- Rust toolchain (rustup installs the `thumbv6m-none-eabi` and
  `thumbv8m.main-none-eabihf` targets automatically via
  `rust-toolchain.toml`)
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x in PATH — optional, only for the `flash.cmd` fallback

## Running the bundled demo

The demo is not part of the crates.io package — it lives in this
repository, and the `cargo rp2040` / `cargo rp2350` commands below are
cargo aliases defined in the repository's `.cargo/config.toml`, so they
work only inside a checkout. Start with:

```sh
git clone https://github.com/ShortArrow/drooling
cd drooling
```

`examples/demo_rp2040.rs` is a complete composite device (CDC serial + reset
interface + LED blink), verified on a Seeed XIAO RP2040 (LED on GPIO25,
active-low) and a Waveshare RP2040-ETH (no user LED; USB and reflash
behavior verified).

First flash (BOOTSEL button required once): enter BOOTSEL mode by holding
BOOT while pressing RESET, then:

```sh
cargo rp2040
```

Every flash after that: same command, no buttons.

`examples/demo_rp2350.rs` is the same composite device for RP2350,
verified on a Waveshare RP2350-GEEK (RP2350A, W25Q128JV 16 MB flash, no
user LED; USB and reflash behavior verified). Build and flash it with:

```sh
cargo rp2350
```

It enumerates as VID:PID `2e8a:0009` ("Pico 2") with the same CDC serial
+ reset interface pair, and Windows binds WinUSB automatically through the
same MS OS 2.0 descriptors.

How the flashing actually works, the other `drool` subcommands, and the
picotool alternative are in [docs/FLASHING.md](docs/FLASHING.md). The USB
interface layout, the reset request and the descriptor design are in
[docs/DESIGN.md](docs/DESIGN.md).

## File structure

```
.
├── src/
│   ├── lib.rs                # crate docs incl. required builder settings
│   ├── picotool_reset.rs     # reset interface UsbClass with MS OS 2.0 handling
│   ├── protocol.rs           # reset request wire-format parsing + tests
│   └── ms_os_20.rs           # BOS / MS OS 2.0 descriptors + structural tests
├── examples/
│   ├── demo_rp2040.rs        # RP2040: CDC serial + reset interface + LED
│   └── demo_rp2350.rs        # RP2350: CDC serial + reset interface
├── tools/
│   ├── drool/                # bundled Rust flasher (nusb reset + PICOBOOT)
│   └── flash.cmd             # picotool fallback (reboot -f -u, then load)
├── .cargo/config.toml        # drool runner + per-chip build config
├── docs/                     # Japanese README, FLASHING, DESIGN, PROTOCOL,
│                             #   CONTRIBUTING, ROADMAP, ADRs, CHANGELOG,
│                             #   and the investigation record
├── variants/                 # earlier experiment binaries (not built)
└── memory/                   # per-chip linker memory layouts
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
