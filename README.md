# drooling

[English](README.md) | [日本語](docs/README.jp.md)

Button-free flashing for Rust firmware on RP2040, using picotool's vendor
reset interface — pure Rust, Pico SDK compatible, works on Windows.

Add the `VendorResetWinUsb` class to your USB composite device and
`picotool reboot -f -u` reboots the running firmware into BOOTSEL mode, no
BOOTSEL button. Windows binds WinUSB automatically via the bundled
BOS / Microsoft OS 2.0 descriptors — no Zadig, no manual driver setup.

## Using from your project

```toml
# Cargo.toml
[dependencies]
drooling = "0.1"
```

```rust
use drooling::vendor_reset_winusb::VendorResetWinUsb;

// after setting up the rp2040-hal USB bus:
let mut serial = SerialPort::new(&usb_bus);
let mut picotool = VendorResetWinUsb::new(&usb_bus);

let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
    .strings(&[StringDescriptors::new(LangID::EN_US)  // EN_US, not EN
        .manufacturer("Raspberry Pi")
        .product("Pico")
        .serial_number("123456")])
    .unwrap()
    .usb_rev(UsbRev::Usb210)   // Windows requests BOS only from >= 0x0210
    .max_packet_size_0(64)     // rp2040-hal enumeration hazard below 18
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

For button-free `cargo run`, copy `tools/flash.cmd` and the `runner` line of
`.cargo/config.toml` into your project.

## Requirements

- Rust toolchain with the `thumbv6m-none-eabi` target
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x in PATH

## Running the bundled demo

`examples/demo.rs` is a complete composite device (CDC serial + reset
interface + LED blink), verified on a Seeed XIAO RP2040 (LED on GPIO25,
active-low on the XIAO).

First flash (BOOTSEL button required once): enter BOOTSEL mode by holding
BOOT while pressing RESET, then:

```sh
cargo run --release --example demo
```

Every flash after that: same command, no buttons. The cargo runner
(`flash.cmd`) sends `picotool reboot -f -u` to the running firmware, waits
for BOOTSEL enumeration, and loads the new binary.

> **Windows note**: picotool rejects single-shot forced commands
> (`picotool load -f`) for RP2040 on Windows, so the runner uses the
> supported two-step flow (`reboot -f -u`, then `load`). On Linux/macOS,
> `picotool load -f` should work directly with this firmware.

## How it works

The demo enumerates as a composite USB device (VID:PID `2e8a:000a`):

| Interface | Class | Purpose |
|-----------|-------|---------|
| 0, 1 | CDC ACM | USB serial port |
| 2 | Vendor (`FF/00/01`) | Pico SDK compatible reset interface |

`picotool reboot -f -u` sends a class request to the vendor interface; the
firmware calls the boot ROM's `reset_to_usb_boot()` and re-enumerates as a
BOOTSEL device (`src/vendor_reset_winusb.rs`).

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
│   ├── vendor_reset_winusb.rs# reset interface UsbClass with MS OS 2.0 handling
│   └── ms_os_20.rs           # BOS / MS OS 2.0 descriptors + structural tests
├── examples/demo.rs          # CDC serial + reset interface + LED
├── tools/flash.cmd           # button-free flash script (reboot -f -u, then load)
├── .cargo/config.toml        # picotool runner + build target
├── docs/                     # Japanese README, CONTRIBUTING, ROADMAP, ADRs,
│                             #   and the investigation record (CONCLUSION.md)
├── variants/                 # earlier experiment binaries (not built)
└── memory.x                  # RP2040 memory layout
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
