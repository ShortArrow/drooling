# ADR 0003: 手書きディスクリプタはホスト側の構造テストで検証する

## Status

Accepted (2026-07-25)

## Context

MS OS 2.0 descriptor set は手書きのバイト配列で、開発中に実際に踏んだ
バグはすべて「長さ・オフセット・包含関係」の構造不整合だった:

- `bFirstInterface` のパッチ位置ずれ(隣の記述子の長さフィールドを破壊)
- subset ヘッダ長の自身ヘッダ分の数え忘れ(子記述子が親領域をはみ出す)

これらは実機 + Windows ETW トレースでしか発見できず、1件の検出に
BOOTSEL ボタン操作を伴う書き込みサイクルが必要だった。Windows の
バリデータは構造不整合のセットを黙って拒否し、列挙全体が失敗する。

## Consequences (先に判明していた制約)

crate は `no_std` だが、ディスクリプタ定義(`ms_os_20`)は依存ゼロの
純データなのでホストでコンパイル・実行できる。ハードウェア依存モジュール
(`vendor_reset_winusb` など)は `cfg(all(target_arch = "arm",
target_os = "none"))` でゲートすればよい。

## Decision

- `lib.rs` でハードウェア依存モジュールを target cfg でゲートし、
  `ms_os_20` は無条件公開する。
- `ms_os_20` に Windows バリデータと同じ規則(各記述子長の整合、
  subset の包含、終端一致)を検査するユニットテストを置く。
- 実行は `cargo test --lib --target <ホストトリプル>`
  (`.cargo/config.toml` が既定ターゲットを thumbv6m にしているため
  明示指定が必要)。CI でも実行する。

## 検証の層構造

1. ホストテスト — 構造不整合(このバグクラスの支配的部分)を秒で検出
2. 実機 + Windows — 列挙成功・WinUSB バインド・`picotool reboot -f -u`
   の疎通(意味的な正しさはここでしか確認できない)
