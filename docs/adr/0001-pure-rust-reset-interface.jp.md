# ADR 0001: picotool reset interface を usb-device 上の純 Rust で実装する

## Status

Accepted (2026-07-25)

## Context

RP2040 の Rust ファームウェアで、picotool による BOOTSEL ボタン不要の
書き込み(vendor reset interface 経由の BOOTSEL 再起動)を実現したい。
Windows では vendor interface に WinUSB が自動バインドされる必要があり、
それには BOS + Microsoft OS 2.0 ディスクリプタが必須。

候補は3つあった:

1. 既存クレート `usbd-picotool-reset` — interface 記述子は正しいが
   MS OS 2.0 ディスクリプタを実装しておらず(v0.3.0、2024-05 以降更新なし)、
   Windows ではデバイスが列挙されない。
2. Pico SDK (TinyUSB) の C 実装を FFI で呼ぶ — 動作するが、CMake での
   静的ライブラリ事前ビルド・リンク調整・C/Rust 初期化順序の管理が必要。
3. `usb-device` の `get_bos_descriptors` と vendor control request で
   BOS + MS OS 2.0 を自前実装 — usb-device 0.3 で API は揃っている。

## Decision

候補3を採用する。`vendor_reset_winusb::VendorResetWinUsb`(UsbClass)が
interface 記述子・BOS platform capability・MS OS 2.0 descriptor set・
reset 要求処理(boot ROM `reset_to_usb_boot`)をすべて持つ。

Windows で列挙に必要な条件は利用側の `UsbDeviceBuilder` 設定に及ぶ
(bcdUSB 0x0210、EP0 64 バイト、`LangID::EN_US`、IAD 複合構成、
usb-device の feature `control-buffer-256`)。詳細な失敗モードと検証記録は
docs/CONCLUSION.md にある。

## Consequences

- C ツールチェーン不要で、依存は usb-device / rp2040-hal / cortex-m のみ。
- ディスクリプタのバイト列は手書きになるため、構造検証は ADR 0003 の
  ホストテストで担保する。
- usb-device の仕様(言語 ID 完全一致など)に列挙成否が依存するため、
  利用側必須設定を crate docstring に集約して明示する。
