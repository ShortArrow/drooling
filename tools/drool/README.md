# drool

Button-free flasher for RP2040 and RP2350: reboots running firmware into
BOOTSEL over the Pico reset interface, writes the ELF over PICOBOOT, and
restarts the application. One command, no picotool, no batch files —
including RP2040 on Windows, where picotool insists on two steps.

```sh
cargo install drool
```

The usual setup is as a cargo runner, so `cargo run` flashes the board:

```toml
# .cargo/config.toml of your firmware project
runner = "drool run"
```

| Command | Effect |
|---------|--------|
| `drool run <ELF>` | Reboot the running device into BOOTSEL if needed, flash, restart |
| `drool flash <ELF> [--no-run]` | Flash a device already in BOOTSEL |
| `drool reboot [--app]` | Reset only: into BOOTSEL, or with `--app` back into the application |

## What the firmware needs

The reboot step finds the device by the Pico SDK reset interface (class
`FF/00/01`, any VID/PID). Pico SDK firmware built with USB stdio has it;
Rust firmware gets it from the
[drooling](https://crates.io/crates/drooling) crate. A device already in
BOOTSEL mode needs nothing — `drool flash` talks straight to the boot ROM.

On Windows the reset interface must be bound to WinUSB; firmware with
Microsoft OS 2.0 descriptors (the SDK's and drooling's both carry them)
gets that binding automatically. On Linux, udev rules are needed for the
reset interface and the BOOTSEL device, or run as root.

The wire protocol and the flashing flow are documented in the
[repository](https://github.com/ShortArrow/drooling): `docs/PROTOCOL.md`
and `docs/FLASHING.md`.

## License

Licensed under either of Apache License 2.0 or MIT, at your option.
