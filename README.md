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

(This repository runs its bundled copy as `cargo run -q -p drool -- run`
instead.) Copying `tools/flash.cmd` and its commented runner line still
works if you would rather stay on picotool.

## Requirements

- Rust toolchain (rustup installs the `thumbv6m-none-eabi` and
  `thumbv8m.main-none-eabihf` targets automatically via
  `rust-toolchain.toml`)
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x in PATH — optional, only for the `flash.cmd` fallback

## Running the bundled demo

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

### How flashing works now

The cargo runner is `drool`, the Rust flasher in `tools/drool`
(`cargo run -q -p drool -- run`). One invocation does the whole cycle: it
finds the running device by its reset interface (class `FF/00/01`, any
VID/PID), reboots it into BOOTSEL, waits for the ROM, erases and writes
over PICOBOOT, reads the first 256 bytes back to verify, and reboots into
the application. A device already sitting in BOOTSEL skips the reset step.
Verified end to end on a Seeed XIAO RP2040 (`2e8a:000a`) and a Waveshare
RP2350-GEEK (`2e8a:0009`), from both entry states.

`drool` also has `reboot [--app]` (reset only, no flashing) and
`flash <ELF> [--no-run]` for flashing without the final restart.

`tools/flash.cmd` remains as a picotool fallback: it exists because
picotool refuses single-shot forced commands (`picotool load -f`) for
RP2040 on Windows and needs the two-step `reboot -f -u`, then `load`
instead. That restriction was picotool policy, not a platform limit —
`drool` flashes RP2040 on Windows in one command.

### RP2350

`examples/demo_rp2350.rs` is the same composite device for RP2350,
verified on a Waveshare RP2350-GEEK (RP2350A, W25Q128JV 16 MB flash, no
user LED; USB and reflash behavior verified). Build and flash it with:

```sh
cargo rp2350
```

It enumerates as VID:PID `2e8a:0009` ("Pico 2") with the same CDC serial
+ reset interface pair, and Windows binds WinUSB automatically through the
same MS OS 2.0 descriptors.

Flashing works exactly as described above — `drool` is the runner for both
chips, and the one-command flow is the same on RP2350 as on RP2040.

> **Linux/macOS note**: `drool` is pure Rust with no batch file involved,
> so it is expected to work on both, but this project has verified it only
> on Windows. On Linux, udev rules are needed for both the reset interface
> of the running device and the BOOTSEL device (or run as root).

Poll requirement: `usb_dev.poll(...)` must run at least every 10 ms while
connected, so keep the main loop tight or poll from a USB interrupt.

VID/PID: the `0x2e8a:0x000a` pair in the example is Raspberry Pi's; fine
for personal boards, but products should use their own VID — picotool
finds the reset interface by its class triple (`FF/00/01`) on third-party
VIDs too.

## How it works

The demo enumerates as a composite USB device (VID:PID `2e8a:000a`):

| Interface | Class | Purpose |
|-----------|-------|---------|
| 0, 1 | CDC ACM | USB serial port |
| 2 | Vendor (`FF/00/01`) | Pico SDK compatible reset interface |

`picotool reboot -f -u` sends a class request to the vendor interface; the
firmware calls the boot ROM's `reset_to_usb_boot()` and re-enumerates as a
BOOTSEL device (`src/picotool_reset.rs`). On RP2350 the firmware calls the
boot ROM's reboot API instead, and the request's GPIO activity pin is
accepted and ignored — that API has no such parameter.

On the host side, `drool` sends that same class request over `nusb` — it
locates the interface by its class triple, so any VID/PID works — and then
speaks PICOBOOT to the boot ROM to erase, write and verify, using the same
protocol picotool does.

For Windows to bind WinUSB to the vendor interface automatically, the device
provides Microsoft OS 2.0 descriptors (`src/ms_os_20.rs`): a BOS platform
capability descriptor plus a descriptor set carrying the `WINUSB` compatible
ID and picotool's device interface GUID
`{bc7398c1-73cd-4cb7-98b8-913a8fca7bf6}`.

## Testing

Host-side structural tests validate the hand-written descriptor byte arrays
(lengths, offsets, containment — the dominant bug class for these):

```sh
cargo test --lib --target x86_64-pc-windows-msvc
```

On-target verification: flash the demo, then check that Windows shows the
device with a serial-number-based instance ID, a COM port, and an error-free
"Reset" interface bound to WinUSB, and that `picotool reboot -f -u` works.

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
├── docs/                     # Japanese README, CONTRIBUTING, ROADMAP, ADRs,
│                             #   CHANGELOG, and the investigation record
├── variants/                 # earlier experiment binaries (not built)
└── memory/                   # per-chip linker memory layouts
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
