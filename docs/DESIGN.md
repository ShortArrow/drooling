# Design

[English](DESIGN.md) | [日本語](DESIGN.jp.md)

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

## Poll requirement

`usb_dev.poll(...)` must run at least every 10 ms while connected, so keep
the main loop tight or poll from a USB interrupt.

## VID/PID

The `0x2e8a:0x000a` pair in the example is Raspberry Pi's; fine for
personal boards, but products should use their own VID — picotool finds the
reset interface by its class triple (`FF/00/01`) on third-party VIDs too.
