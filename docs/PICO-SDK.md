# Relationship to the Pico SDK

[English](PICO-SDK.md) | [日本語](PICO-SDK.jp.md)

Everything this crate does exists first in the C world, provided by the
official Pico SDK. This page maps the two worlds onto each other.

## What the SDK does in C/C++

In an SDK project the reset interface is part of the `pico_stdio_usb`
component. The user writes two lines:

```cmake
pico_enable_stdio_usb(my_target 1)   # CMakeLists.txt
```

```c
stdio_init_all();                     // top of main()
```

and the SDK pulls in TinyUSB and builds the whole composite device — CDC
serial for stdio, the vendor reset interface, and the Microsoft OS 2.0
descriptors that make Windows bind WinUSB — without the user writing any
USB code. The reset interface is enabled by default whenever USB stdio
is (`PICO_STDIO_USB_ENABLE_RESET_VIA_VENDOR_INTERFACE`).

The automation is conditional, though: it rides on USB stdio. A C
program that configures TinyUSB by hand instead of using `stdio_usb`
does not get the reset interface for free and must integrate it itself.

## What drooling is

`drooling` is that same functionality for the Rust `usb-device`
ecosystem, where no working equivalent existed. On the wire it is
identical to the SDK implementation — the same class requests, the same
descriptors, the same device interface GUID — so host tools cannot tell
a drooling firmware from an SDK one.

One deliberate difference in shape: in the SDK the reset interface is a
rider on stdio; here it is an independent USB class (`PicotoolReset`)
that you add to whatever composite device you are building. Pairing it
with CDC serial, as the demos do, is a choice rather than a requirement.

## The pairing table

| | C world | Rust world |
|---|---|---|
| Firmware side | Pico SDK (`pico_stdio_usb`) | `drooling` |
| Host side | picotool | `drool` (or picotool — the protocol is shared) |
