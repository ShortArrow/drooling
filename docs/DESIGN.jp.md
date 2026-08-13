# 設計

[English](DESIGN.md) | [日本語](DESIGN.jp.md)

## 動作原理

デモは USB 複合デバイス(VID:PID `2e8a:000a`)として列挙される:

| Interface | Class | 役割 |
|-----------|-------|------|
| 0, 1 | CDC ACM | USB シリアルポート |
| 2 | Vendor (`FF/00/01`) | Pico SDK 互換 reset interface |

`picotool` も `drool` も、vendor interface へ reset 要求を送ることで
ボードを再起動させる。ファームウェアはそれに応えて boot ROM へ制御を渡し
(`src/picotool_reset.rs`)、デバイスは一度バスから消えて BOOTSEL デバイス
として戻ってくる。以降ホストは ROM と会話して消去・書き込み・検証を行う。
ファームウェアは RP2040 と RP2350 それぞれの ROM の入口を使うため、
1つの interface で両チップをカバーできる。

デバイスは Microsoft OS 2.0 ディスクリプタも持つ(`src/ms_os_20.rs`)。
その役割は、Zadig も手動ドライバ導入もなしに Windows が vendor interface へ
WinUSB をバインドするようにすること、それだけ。

リクエストコード・`wValue` のレイアウト・転送単位の手順は
[PROTOCOL.jp.md](PROTOCOL.jp.md) にある。

## poll 要件

接続中は `usb_dev.poll(...)` を最低 10ms ごとに呼ぶ必要がある。
メインループを軽く保つか、USB 割り込みから poll すること。

## VID/PID

例の `0x2e8a:0x000a` は Raspberry Pi のもの。個人のボードでは問題ないが、
製品では自前の VID を使うこと。それで壊れるものはない — picotool も
`drool` も、サードパーティ VID でも class triple で reset interface を
発見する。
