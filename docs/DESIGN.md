# Design

[English](DESIGN.md) | [日本語](DESIGN.jp.md)

How this maps onto what the Pico SDK provides in the C world is in
[PICO-SDK.md](PICO-SDK.md).

## How it works

The demo enumerates as a composite USB device (VID:PID `2e8a:000a`):

| Interface | Class | Purpose |
|-----------|-------|---------|
| 0, 1 | CDC ACM | USB serial port |
| 2 | Vendor (`FF/00/01`) | Pico SDK compatible reset interface |

Both `picotool` and `drool` send a reset request to the vendor
interface. The firmware hands control to the boot ROM
(`src/picotool_reset.rs`); the device drops off the bus, comes back as a
BOOTSEL device, and from then on the host's counterpart is the ROM,
which handles erase, write and verify. The firmware uses the RP2040 and RP2350 ROM entry points
respectively, so one interface covers both chips.

The device also carries Microsoft OS 2.0 descriptors (`src/ms_os_20.rs`),
whose only job is to get Windows to bind WinUSB to the vendor interface
without Zadig or a manual driver install.

Request codes, the `wValue` layout and the transfer-by-transfer sequence are
in [PROTOCOL.md](PROTOCOL.md).

## Poll requirement

`usb_dev.poll(...)` must run at least every 10 ms while connected, so keep
the main loop tight or poll from a USB interrupt.

## VID/PID

The `0x2e8a:0x000a` pair in the example is Raspberry Pi's; fine for
personal boards, but products should use their own VID. Nothing breaks:
both picotool and `drool` find the reset interface by its class triple on
third-party VIDs too.
