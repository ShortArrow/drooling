# Roadmap

[English](ROADMAP.md) | [日本語](ROADMAP.jp.md)

優先度順ではなく領域別。確定した設計は [adr/](adr/) にある。

## クロスプラットフォーム検証

ファームウェア側はプラットフォーム非依存だが、エンドツーエンドの検証は
Windows のみ。

- Linux / macOS でワンショット `picotool load -f` を検証する
- プラットフォーム別の runner 設定を文書化する(2段階の `flash.cmd`
  フローが必要なのは Windows だけ)

## RP2350 対応

- `vendor_reset_winusb` を `rp235x-hal` に移植する(boot ROM API が異なる)
- RP2350 は Windows での picotool 制限が少ないため、書き込みフローを再検討

## デモの近代化

- `examples/demo.rs` の `static mut` な USB バスアロケータパターンを
  警告なしのイディオムに置き換える
- reset interface の `embassy-usb` 版を検討する(embassy は `msos`
  モジュールで MS OS 2.0 を第一級サポート)

## ホストツールの all Rust 化

- ホスト側の picotool (C++ 製) を Rust ツールで置き換える: vendor reset
  要求を `nusb` で送り、書き込みは PICOBOOT プロトコルを直接実装して、
  書き込みフローを単一の Rust バイナリに畳む

## リポジトリ整理

- 純 Rust 実装が十分に安定した時点で `variants/` を撤去する
