# Architecture Decision Records

drooling の設計判断の記録。各 ADR は現状の実装を説明する判断を示す。
変更は新しい ADR で supersede し、対象が実装から消えた ADR は削除する。

| ADR | タイトル | Status |
| --- | -------- | ------ |
| [0000](0000-documentation-policy.jp.md) | ドキュメンテーション方針 | Accepted |
| [0001](0001-pure-rust-reset-interface.jp.md) | picotool reset interface を usb-device 上の純 Rust で実装する | Accepted |
| [0002](0002-windows-two-step-flash.jp.md) | Windows では reboot→load の2段階フローを flash.cmd で自動化する | Accepted |
| [0003](0003-host-side-descriptor-tests.jp.md) | 手書きディスクリプタはホスト側の構造テストで検証する | Accepted |
| [0004](0004-library-boundary.jp.md) | ライブラリ境界 — rp2040-hal 依存・BSP 非依存・デモは examples/ | Accepted |
| [0005](0005-chip-selection-features.jp.md) | チップ (RP2040 / RP2350) は相互排他な cargo feature で選択する | Accepted |
