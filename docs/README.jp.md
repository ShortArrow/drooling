# drooling

[English](../README.md) | [日本語](README.jp.md)

RP2040 の Rust ファームウェアを、picotool の vendor reset interface 経由で
ボタン操作なしに書き込むためのクレート。純 Rust・Pico SDK 互換・Windows 対応。

`PicotoolReset` クラスを USB 複合デバイスに追加すると、
`picotool reboot -f -u` で実行中ファームを BOOTSEL モードへ再起動できる
(BOOTSEL ボタン不要)。同梱の BOS / Microsoft OS 2.0 ディスクリプタにより
Windows は WinUSB を自動バインドする — Zadig や手動ドライバ導入は不要。

## 他プロジェクトからの使い方

```toml
# Cargo.toml
[dependencies]
drooling = "0.1"
```

```rust
use drooling::PicotoolReset;

// rp2040-hal の USB バス構築後:
let mut serial = SerialPort::new(&usb_bus);
let mut picotool = PicotoolReset::new(&usb_bus);

let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2e8a, 0x000a))
    .strings(&[StringDescriptors::new(LangID::EN_US)  // EN ではなく EN_US
        .manufacturer("Raspberry Pi")
        .product("Pico")
        .serial_number("123456")])
    .unwrap()
    .usb_rev(UsbRev::Usb210)   // Windows は bcdUSB >= 0x0210 でのみ BOS を要求
    .max_packet_size_0(64)     // rp2040-hal は EP0 が小さいと列挙不具合
    .unwrap()
    .composite_with_iads()
    .build();

loop {
    usb_dev.poll(&mut [&mut serial, &mut picotool]);
    // アプリケーション処理
}
```

上記でマークした4つの Builder 設定は Windows では必須(どれが欠けても
列挙失敗、もしくは WinUSB が付かない)。`usb-device` の必須 feature
`control-buffer-256` は、このクレートへの依存で自動的に有効になる。

ボタンレス `cargo run` にするには、`tools/flash.cmd` と `.cargo/config.toml` の
`runner` 行を自分のプロジェクトへコピーする。

## 必要なもの

- Rust toolchain(`thumbv6m-none-eabi` ターゲット)
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x(PATH が通っていること)

## 同梱デモの実行

`examples/demo.rs` は完全な複合デバイス(CDC シリアル + reset interface +
LED 点滅)。Seeed XIAO RP2040 で検証済み(LED は GPIO25、アクティブ Low)。

初回のみ BOOTSEL ボタンが必要: BOOT を押しながら RESET を押して
BOOTSEL モードにしてから:

```sh
cargo run --release --example demo
```

2回目以降は同じコマンドだけ、ボタン不要。cargo runner(`flash.cmd`)が
実行中ファームに `picotool reboot -f -u` を送り、BOOTSEL の列挙を待って
新しいバイナリを書き込む。

> **Windows の注意**: picotool は RP2040 + Windows でワンショットの
> forced command(`picotool load -f`)を拒否するため、runner はサポート
> されている2段階フロー(`reboot -f -u` → `load`)を使う。Linux/macOS
> では `picotool load -f` が直接使えるはず。

## 動作原理

デモは USB 複合デバイス(VID:PID `2e8a:000a`)として列挙される:

| Interface | Class | 役割 |
|-----------|-------|------|
| 0, 1 | CDC ACM | USB シリアルポート |
| 2 | Vendor (`FF/00/01`) | Pico SDK 互換 reset interface |

`picotool reboot -f -u` は vendor interface へ class request を送り、
ファームウェアが boot ROM の `reset_to_usb_boot()` を呼んで BOOTSEL
デバイスとして再列挙する(`src/picotool_reset.rs`)。

Windows が vendor interface に WinUSB を自動バインドできるよう、
Microsoft OS 2.0 ディスクリプタを提供する(`src/ms_os_20.rs`):
BOS platform capability と、`WINUSB` compatible ID および picotool の
device interface GUID `{bc7398c1-73cd-4cb7-98b8-913a8fca7bf6}` を含む
ディスクリプタセット。

## テスト

手書きディスクリプタのバイト配列(長さ・オフセット・包含関係 — この種の
実装で支配的なバグクラス)をホスト側で構造検証する:

```sh
cargo test --lib --target x86_64-pc-windows-msvc
```

実機検証: デモを書き込み後、Windows でシリアル番号ベースのインスタンス
ID・COM ポート・エラーなしで WinUSB にバインドされた「Reset」interface を
確認し、`picotool reboot -f -u` が通ること。

## ファイル構成

```
.
├── src/
│   ├── lib.rs                # クレートドキュメント (必須 Builder 設定含む)
│   ├── picotool_reset.rs# reset interface UsbClass (MS OS 2.0 応答含む)
│   └── ms_os_20.rs           # BOS / MS OS 2.0 ディスクリプタ + 構造テスト
├── examples/demo.rs          # CDC シリアル + reset interface + LED
├── tools/flash.cmd           # ボタンレス書き込みスクリプト (reboot -f -u → load)
├── .cargo/config.toml        # picotool runner + ビルドターゲット
├── docs/                     # 日本語 README, CONTRIBUTING, ROADMAP, ADR,
│                             #   調査記録 (CONCLUSION.md)
├── variants/                 # 過去の実験バイナリ (ビルド対象外)
└── memory.x                  # RP2040 メモリレイアウト
```

## ライセンス

以下のいずれかのライセンスを選択できる:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))
