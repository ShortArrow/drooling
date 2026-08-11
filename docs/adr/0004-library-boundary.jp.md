# ADR 0004: ライブラリ境界 — rp2040-hal 依存・BSP 非依存・デモは examples/

## Status

Accepted (2026-07-25)

## Context

他プロジェクトのファームウェアから USB クラスとして組み込んで使うため、
依存を最小化したライブラリ境界が要る。ボードは Raspberry Pi Pico に
限らない(検証機は Seeed XIAO RP2040 と Waveshare RP2040-ETH)。

## Decision

- **lib の依存は `rp2040-hal` + `usb-device` + `cortex-m` のみ。**
  boot ROM 呼び出しは `rp2040_hal::rom_data` を使い、特定ボードの BSP に
  依存しない。
- **デモは `examples/demo.rs`**(`cargo run --release --example demo`)。
  BSP(`rp-pico`)・`usbd-serial` 等のデモ専用依存は `[dev-dependencies]`
  に置き、利用側の依存グラフを汚さない。
- **crates.io パッケージは `src/` + README + LICENSE のみ**
  (Cargo.toml の include リスト)。examples はホストでコンパイルできない
  (cortex-m-rt エントリポイントを持つ)ため、`cargo publish` の検証
  ビルドを通すには同梱しない。使用例は README と docstring が担う。
- **利用側の必須 Builder 設定**(EN_US / Usb210 / EP0=64 / IAD)は crate
  レベル docstring に集約する。ライブラリ側では強制できないため、
  ドキュメンテーションで契約を示す。

## Consequences

- 消費側は `drooling = { path = ... }` または crates.io 依存のみで使える
  (パスビルドで実証済み)。
