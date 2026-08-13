# Flashing

[English](FLASHING.md) | [日本語](FLASHING.jp.md)

The first flash of a fresh board needs the BOOTSEL button once (hold BOOT
while pressing RESET); every flash after that needs no buttons.

## The drool flow

`drool` is the Rust flasher developed in this repository (`tools/drool`).
The setup depends on where you run it:

- **Inside a checkout of this repository** nothing needs installing: the
  configured runner, `cargo run -q -p drool -- run`, builds and runs the
  bundled copy on the spot.
- **In your own project** install it once with `cargo install drool`,
  then set `runner = "drool run"` in your `.cargo/config.toml`.

`run <ELF>` handles everything in one invocation, and works the same on a
device that is already sitting in BOOTSEL. `drool` also has
`reboot [--app]` (reset only, no flashing) and `flash <ELF> [--no-run]`
for flashing without the final restart.

What actually happens on the wire is in [PROTOCOL.md](PROTOCOL.md).

## Flashing with picotool instead

Everything here works with picotool v2.x as well — the firmware speaks
the Pico SDK protocol, so any picotool release can drive it. Point the
`runner` in `.cargo/config.toml` at one of these instead of `drool run`:

```toml
# Linux / macOS, and Windows with RP2350: one command
runner = "picotool load -f -x -t elf"

# Windows with RP2040: two steps, wrapped in the bundled batch file
runner = "./tools/flash.cmd"
```

The batch file is not part of the crates.io package; copy
[`tools/flash.cmd`](https://github.com/ShortArrow/drooling/blob/main/tools/flash.cmd)
out of this repository into your project at `tools/flash.cmd`, or create
it there with this content (identical to the copy in this repository):

```bat
@echo off
rem Button-free flash for RP2040 on Windows.
rem
rem picotool on Windows rejects single-shot forced commands for RP2040
rem ("picotool load -f"), so this script uses the supported two-step flow:
rem reboot the running firmware into BOOTSEL via its vendor reset interface,
rem then load. Works from both application mode and BOOTSEL mode.

picotool reboot -f -u >nul 2>&1

for /l %%i in (1,1,10) do (
  picotool load -x -t elf %1 && exit /b 0
  ping -n 2 127.0.0.1 >nul
)
echo picotool load failed after 10 attempts 1>&2
exit /b 1
```

The batch file exists because picotool refuses single-shot forced
commands (`picotool load -f`) for RP2040 on Windows and wants
`picotool reboot -f -u` followed by `picotool load` instead; the script
runs the two in order and retries the load while the ROM enumerates. That
restriction is picotool policy rather than a platform limit, which is
why `drool` flashes RP2040 on Windows in one command.

Without a cargo runner at all, the same two steps by hand:

```sh
picotool reboot -f -u                    # running firmware -> BOOTSEL
picotool load -x -t elf <path-to-elf>    # flash and run
```

> **Linux/macOS note**: `drool` is pure Rust with no batch file involved,
> so it is expected to work on both, but this project has verified it only
> on Windows. On Linux, udev rules are needed for both the reset interface
> of the running device and the BOOTSEL device (or run as root).
