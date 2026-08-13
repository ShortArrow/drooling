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

(本リポジトリ自身は同梱コピーを `cargo run -q -p drool -- run` で
使っている。)

picotool のままにしたい場合:
[`tools/flash.cmd`](https://github.com/ShortArrow/drooling/blob/main/tools/flash.cmd)
を本リポジトリから自分のプロジェクトの `tools/flash.cmd` へコピーし
(crates.io のパッケージには含まれていない)、
`runner = "./tools/flash.cmd"` を設定する。プラットフォーム別の
picotool runner 行(バッチ不要のワンコマンド版を含む)は後述の
picotool 節にある。

## 必要なもの

- Rust toolchain(`thumbv6m-none-eabi` と `thumbv8m.main-none-eabihf`
  ターゲットは `rust-toolchain.toml` により rustup が自動導入)
- `flip-link`: `cargo install flip-link`
- `picotool` v2.x(PATH が通っていること)— 任意、`flash.cmd`
  フォールバック用

## 同梱デモの実行

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

### 現在の書き込みの仕組み

cargo runner は `tools/drool` の Rust 製書き込みツール `drool`
(`cargo run -q -p drool -- run`)。1回の実行で全工程を行う: reset
interface(class `FF/00/01`、VID/PID は問わない)で実行中デバイスを探し、
BOOTSEL へ再起動し、ROM の起動を待ち、PICOBOOT で消去・書き込みを行い、
先頭 256 バイトを読み戻して検証し、アプリケーションへ再起動する。
既に BOOTSEL 状態のデバイスでは reset 手順を飛ばす。Seeed XIAO RP2040
(`2e8a:000a`)と Waveshare RP2350-GEEK(`2e8a:0009`)で、どちらの開始
状態からもエンドツーエンドで検証済み。

`drool` には `reboot [--app]`(書き込みなしの再起動のみ)と
`flash <ELF> [--no-run]`(最後の再起動を行わない書き込み)もある。

#### picotool で書き込む場合

ここまでの内容は picotool v2.x でもそのまま成立する — ファームウェアは
Pico SDK のプロトコルを話すため、picotool から普通に駆動できる。
`.cargo/config.toml` の `runner` を `drool run` の代わりに次のいずれかに
向ければよい:

```toml
# Linux / macOS、および Windows + RP2350: 1コマンド
runner = "picotool load -f -x -t elf"

# Windows + RP2040: 2段階を同梱バッチにまとめたもの
runner = "./tools/flash.cmd"
```

このバッチは crates.io のパッケージには含まれていないので、自分の
プロジェクトに `tools/flash.cmd` として以下の内容で作成する
(本リポジトリのコピーと同一):

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

このバッチが存在するのは、picotool が RP2040 + Windows でワンショットの
forced command(`picotool load -f`)を拒否し、`picotool reboot -f -u` →
`picotool load` の2段階を要求するため。スクリプトは両者を実行し、ROM が
列挙されるまで load をリトライする。これはプラットフォームの制約ではなく
picotool の方針であり、`drool` は Windows の RP2040 でも1コマンドで
書き込む。

cargo runner を使わず手で叩く場合も同じ2段階:

```sh
picotool reboot -f -u                    # 実行中ファーム -> BOOTSEL
picotool load -x -t elf <ELF のパス>     # 書き込んで実行
```

### RP2350

`examples/demo_rp2350.rs` は同じ複合デバイスの RP2350 版。
Waveshare RP2350-GEEK(RP2350A、W25Q128JV 16MB フラッシュ、ユーザー LED
なし、USB と再書き込み動作を検証)で検証済み。ビルドと書き込み:

```sh
cargo rp2350
```

VID:PID `2e8a:0009`(「Pico 2」)として列挙され、CDC シリアル + reset
interface の構成は同じ。Windows は同じ MS OS 2.0 ディスクリプタにより
WinUSB を自動バインドする。

書き込みは上記のとおり — `drool` は両チップ共通の runner で、
1コマンドのフローは RP2350 でも RP2040 と同じ。

> **Linux/macOS の注意**: `drool` はバッチファイルを介さない純 Rust
> 実装のため両方で動作するはずだが、本プロジェクトでの検証は Windows
> のみ。Linux では実行中デバイスの reset interface と BOOTSEL デバイスの
> 両方に udev rules が必要(または root で実行)。

poll 要件: 接続中は `usb_dev.poll(...)` を最低 10ms ごとに呼ぶ必要が
ある。メインループを軽く保つか、USB 割り込みから poll すること。

VID/PID: 例の `0x2e8a:0x000a` は Raspberry Pi のもの。個人のボードでは
問題ないが、製品では自前の VID を使うこと — picotool はサードパーティ
VID でも class triple(`FF/00/01`)で reset interface を発見する。

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
├── docs/                     # 日本語 README, CONTRIBUTING, ROADMAP, ADR,
│                             #   CHANGELOG, 調査記録 (CONCLUSION.md)
├── variants/                 # 過去の実験バイナリ (ビルド対象外)
└── memory/                   # チップ別リンカメモリレイアウト
```

## ライセンス

以下のいずれかのライセンスを選択できる:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))
