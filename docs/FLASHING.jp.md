# 書き込み

[English](FLASHING.md) | [日本語](FLASHING.jp.md)

新品のボードへの初回書き込みだけは BOOTSEL ボタンが必要(BOOT を押しながら
RESET を押す)。2回目以降はボタン不要。

## drool のフロー

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

## picotool で書き込む場合

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

このバッチは crates.io のパッケージには含まれていないので、
[`tools/flash.cmd`](https://github.com/ShortArrow/drooling/blob/main/tools/flash.cmd)
を本リポジトリから自分のプロジェクトの `tools/flash.cmd` へコピーするか、
以下の内容で作成する(本リポジトリのコピーと同一):

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

> **Linux/macOS の注意**: `drool` はバッチファイルを介さない純 Rust
> 実装のため両方で動作するはずだが、本プロジェクトでの検証は Windows
> のみ。Linux では実行中デバイスの reset interface と BOOTSEL デバイスの
> 両方に udev rules が必要(または root で実行)。
