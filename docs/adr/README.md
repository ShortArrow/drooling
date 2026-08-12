# Architecture Decision Records

Design decisions of drooling, each explaining a decision behind the
current implementation. A change supersedes an ADR with a new one; an
ADR whose subject has left the implementation is removed.

English editions have not been written yet. The rows below already
link to their future `NNNN-slug.md` names, so each link comes alive as
its English edition lands. Until then the records exist only in
Japanese (`NNNN-slug.jp.md`, indexed in `README.jp.md`).

| ADR | Title | Status |
| --- | ----- | ------ |
| [0000](0000-documentation-policy.md) | Documentation policy | Accepted |
| [0001](0001-pure-rust-reset-interface.md) | Implement the picotool reset interface in pure Rust on usb-device | Accepted |
| [0002](0002-windows-two-step-flash.md) | Automate the Windows reboot-then-load flow with flash.cmd | Accepted |
| [0003](0003-host-side-descriptor-tests.md) | Validate hand-written descriptors with host-side structural tests | Accepted |
| [0004](0004-library-boundary.md) | Library boundary: rp2040-hal dependency, BSP-free, demo in examples/ | Accepted |
| [0005](0005-chip-selection-features.md) | Select the chip (RP2040 / RP2350) with mutually exclusive cargo features | Accepted |
