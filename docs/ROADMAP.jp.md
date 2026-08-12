# Roadmap

[English](ROADMAP.md) | [日本語](ROADMAP.jp.md)

優先度順ではなく領域別。確定した設計は [adr/](adr/) にある。

## クロスプラットフォーム検証

ファームウェア側はプラットフォーム非依存だが、エンドツーエンドの検証は
Windows のみ。

- Linux / macOS で `drool` をエンドツーエンドで検証する
- Linux で reset interface と BOOTSEL デバイスの双方に必要な udev rules
  を文書化する

## RP2350 の RISC-V 対応

- RP2350 のもう一方のアーキテクチャ `riscv32imac-unknown-none-elf` 向けに
  デモをビルドして検証する(ARM (thumbv8m) 版は完了済み)

## デモの近代化

- `examples/demo_rp2040.rs` の `static mut` な USB バスアロケータパターンを
  警告なしのイディオムに置き換える
- reset interface の `embassy-usb` 版を検討する(embassy は `msos`
  モジュールで MS OS 2.0 を第一級サポート)

## drool の公開

- `drool` を crates.io へ公開する: 初回は手動で publish し、その後
  Trusted Publishing とリリースワークフローに組み込む(ビルド済み
  バイナリの配布はその後でよい)

## リポジトリ整理

- 純 Rust 実装が十分に安定した時点で `variants/` を撤去する
