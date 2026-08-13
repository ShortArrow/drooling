# 設計

[English](DESIGN.md) | [日本語](DESIGN.jp.md)

## 動作原理

デモは USB 複合デバイス(VID:PID `2e8a:000a`)として列挙される:

| Interface | Class | 役割 |
|-----------|-------|------|
| 0, 1 | CDC ACM | USB シリアルポート |
| 2 | Vendor (`FF/00/01`) | Pico SDK 互換 reset interface |

`picotool reboot -f -u` は vendor interface へ class request を送り、
ファームウェアが boot ROM の `reset_to_usb_boot()` を呼んで BOOTSEL
デバイスとして再列挙する(`src/picotool_reset.rs`)。RP2350 では代わりに
boot ROM の reboot API を呼び、要求に含まれる GPIO アクティビティピンは
受け取った上で無視する — この API にはそのパラメータがない。

ホスト側では `drool` が同じ class request を `nusb` 経由で送り
(interface は class triple で探すため VID/PID は問わない)、その後
boot ROM に対して PICOBOOT を話して消去・書き込み・検証を行う —
picotool と同じプロトコル。

Windows が vendor interface に WinUSB を自動バインドできるよう、
Microsoft OS 2.0 ディスクリプタを提供する(`src/ms_os_20.rs`):
BOS platform capability と、`WINUSB` compatible ID および picotool の
device interface GUID `{bc7398c1-73cd-4cb7-98b8-913a8fca7bf6}` を含む
ディスクリプタセット。

## poll 要件

接続中は `usb_dev.poll(...)` を最低 10ms ごとに呼ぶ必要がある。
メインループを軽く保つか、USB 割り込みから poll すること。

## VID/PID

例の `0x2e8a:0x000a` は Raspberry Pi のもの。個人のボードでは問題ないが、
製品では自前の VID を使うこと — picotool はサードパーティ VID でも
class triple(`FF/00/01`)で reset interface を発見する。
