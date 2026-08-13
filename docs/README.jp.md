# drooling

[English](../README.md) | [日本語](README.jp.md)

RP2040 の Rust ファームウェアを、picotool の vendor reset interface 経由で
ボタン操作なしに書き込むためのクレート。純 Rust・Pico SDK 互換・Windows 対応。

`PicotoolReset` クラスを USB 複合デバイスに追加すると、
`picotool reboot -f -u` で実行中ファームを BOOTSEL モードへ再起動できる
(BOOTSEL ボタン不要)。同梱の BOS / Microsoft OS 2.0 ディスクリプタにより
Windows は WinUSB を自動バインドする — Zadig や手動ドライバ導入は不要。

名前の由来: USB Type-C の口は、よだれを垂らしそうなスライムの口に見える。
その口から漏れ出てくるのは操作 — ボタンを押さなくても再起動・書き込みが
よだれのように USB ポートから滲み出てくる。

## 他プロジェクトからの使い方

```toml
# Cargo.toml
[dependencies]
# RP2040
drooling = { version = "0.2", features = ["rp2040"] }

# RP2350
drooling = { version = "0.2", features = ["rp2350"] }
```

チップの feature は相互排他で既定は無いため、どちらか一方を必ず選ぶ。

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
    .max_packet_size_0(64)     // rp2040-hal は EP0 が 18 バイト未満だと列挙不具合
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

ボタンレス `cargo run` にするには、書き込みツールを導入して `runner` を
向けるだけ:

```sh
cargo install drool
```

```toml
# .cargo/config.toml
runner = "drool run"
```

本リポジトリ自身は同梱コピーを `cargo run -q -p drool -- run` で使っている。

picotool を使う場合の選択肢とプラットフォーム別の注意は
[FLASHING.jp.md](FLASHING.jp.md) にある。

## 必要なもの

- Rust toolchain(`thumbv6m-none-eabi` と `thumbv8m.main-none-eabihf`
  ターゲットは `rust-toolchain.toml` により rustup が自動導入)
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x(PATH が通っていること)— 任意、`flash.cmd`
  フォールバック用

## 同梱デモの実行

デモは crates.io のパッケージには含まれておらず、このリポジトリの中に
ある。後述の `cargo rp2040` / `cargo rp2350` はリポジトリの
`.cargo/config.toml` に定義された cargo エイリアスで、**チェックアウトの
中でしか動かない**。まずは:

```sh
git clone https://github.com/ShortArrow/drooling
cd drooling
```

`examples/demo_rp2040.rs` は完全な複合デバイス(CDC シリアル + reset interface +
LED 点滅)。Seeed XIAO RP2040(LED は GPIO25、アクティブ Low)と
Waveshare RP2040-ETH(ユーザー LED なし、USB と再書き込み動作を検証)で
検証済み。

初回のみ BOOTSEL ボタンが必要: BOOT を押しながら RESET を押して
BOOTSEL モードにしてから:

```sh
cargo rp2040
```

2回目以降は同じコマンドだけ、ボタン不要。

`examples/demo_rp2350.rs` は同じ複合デバイスの RP2350 版。
Waveshare RP2350-GEEK(RP2350A、W25Q128JV 16MB フラッシュ、ユーザー LED
なし、USB と再書き込み動作を検証)で検証済み。ビルドと書き込み:

```sh
cargo rp2350
```

VID:PID `2e8a:0009`(「Pico 2」)として列挙され、CDC シリアル + reset
interface の構成は同じ。Windows は同じ MS OS 2.0 ディスクリプタにより
WinUSB を自動バインドする。

書き込みの実際の仕組み、`drool` の他のサブコマンド、picotool を使う場合は
[FLASHING.jp.md](FLASHING.jp.md) にある。USB interface の構成・reset
要求・ディスクリプタ設計は [DESIGN.jp.md](DESIGN.jp.md) にある。

## ファイル構成

```
.
├── src/
│   ├── lib.rs                # クレートドキュメント (必須 Builder 設定含む)
│   ├── picotool_reset.rs     # reset interface UsbClass (MS OS 2.0 応答含む)
│   ├── protocol.rs           # reset 要求のワイヤ形式パース + テスト
│   └── ms_os_20.rs           # BOS / MS OS 2.0 ディスクリプタ + 構造テスト
├── examples/
│   ├── demo_rp2040.rs        # RP2040: CDC シリアル + reset interface + LED
│   └── demo_rp2350.rs        # RP2350: CDC シリアル + reset interface
├── tools/
│   ├── drool/                # 同梱の Rust 書き込みツール (nusb reset + PICOBOOT)
│   └── flash.cmd             # picotool フォールバック (reboot -f -u → load)
├── .cargo/config.toml        # drool runner + チップ別ビルド設定
├── docs/                     # 日本語 README, FLASHING, DESIGN,
│                             #   CONTRIBUTING, ROADMAP, ADR, CHANGELOG,
│                             #   調査記録 (CONCLUSION.md)
├── variants/                 # 過去の実験バイナリ (ビルド対象外)
└── memory/                   # チップ別リンカメモリレイアウト
```

## ライセンス

以下のいずれかのライセンスを選択できる:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))
