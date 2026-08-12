# ADR 0006: ホスト側書き込みツール drool を同梱する — nusb で reset、PICOBOOT で書き込み

## Status

Accepted (2026-08-13)

## Context

書き込みフローのホスト側は C++ 製の picotool に依存していた。picotool は
RP2040 + Windows でワンショットの forced command(`picotool load -f`)を
拒否するため、この組み合わせだけ2段階フロー(`reboot -f -u` → `load`)を
バッチファイルで自動化していた。これは Windows の USB スタックの制約では
なく picotool 側の方針であり、PICOBOOT を直接話せば回避できる。

目標は、全プラットフォーム・両チップで `cargo run` 一発、かつホスト側も
Rust で完結させること。バッチファイルへの依存は Windows 以外で runner を
差し替える必要を生み、外部バイナリ(picotool)の PATH 導入も前提になる。

## Decision

- **workspace member `tools/drool` としてホストツールを実装する。**
  サブコマンドは `reboot [--app]` / `flash <ELF> [--no-run]` / `run <ELF>`。
  `run` は「実行中デバイスの発見 → BOOTSEL 再起動 → ROM 待ち → 消去・書き込み
  → 先頭 256 バイトの読み戻し検証 → アプリケーションへ再起動」を通しで行う。
  既に BOOTSEL 状態なら reset 手順を飛ばす。
- **デバイス発見と reset は `nusb`。** reset interface は class triple
  (`FF/00/01`)で探し、VID/PID には依存しない — サードパーティ VID の
  ボードでもそのまま動く。送る class request は picotool と同じもの。
- **BOOTSEL 側の書き込みは `picoboot` クレート**(PICOBOOT プロトコル)。
- **`tokio` は `rt` feature のみ有効にする。** nusb の既定 tokio バックエンドは
  await を `spawn_blocking` 経由で解決するため、ランタイムコンテキストが
  無いとパニックする。この要件はリグレッションテストで固定する。
- **ELF → 書き込みプランの生成は純粋モジュールに分離し、仕様をテストで
  固定する。** ページはセグメント開始アドレスを 256 バイト境界へ切り下げて
  合成し、隙間は消去済みフラッシュを表す `0xFF` で埋め、連続ページは1つの
  チャンクに結合し、重なるセグメントは拒否する。USB を必要としないため
  ホストで CI にかけられる。
- **runner は `cargo run -q -p drool -- run`** を両チップのターゲットに
  設定する。`tools/flash.cmd` は picotool フォールバックとして残し、
  `.cargo/config.toml` にコメント行で併記する。

## Consequences

- 両チップとも、Windows の RP2040 を含めて1コマンドで書き込める。
  初回書き込みに BOOTSEL ボタンが必要な点は変わらない。
- picotool は任意になる。必要なのは `flash.cmd` フォールバックを使う場合だけ。
- Linux/macOS は純 Rust ゆえ動作するはずだが実機未検証。Linux では
  実行中デバイスの reset interface と BOOTSEL デバイスの両方に udev rules
  が要る。
- drool は crates.io 未公開。利用側は当面このリポジトリからビルドする。
  公開後は `cargo install drool` と `runner = "drool run"` だけで済むように
  なる。
