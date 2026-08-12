# ADR 0002: Windows では reboot→load の2段階フローを flash.cmd で自動化する

## Status

Accepted (2026-07-25) — 既定 runner としての役割は 0006 で drool に置き換え。
flash.cmd は picotool フォールバックとして存続。

## Context

picotool は RP2040 + Windows の組み合わせで、ワンショットの forced command
(`picotool load -f`)を意図的に拒否する:

```
ERROR: Forced commands do not work with RP2040 on Windows -
you can force reboot into BOOTSEL mode via 'picotool reboot -f -u' instead.
```

これは Windows の USB mass storage ドライバの挙動に起因する picotool 側の
制限で、ファームウェア側では解決できない。一方 `picotool reboot -f -u`
(BOOTSEL へ再起動)と、BOOTSEL 状態への `picotool load` は個別には動く。

## Decision

`flash.cmd` が「`picotool reboot -f -u`(失敗は無視)→ `picotool load -x
-t elf` を1秒間隔で最大10回リトライ」を実行し、`.cargo/config.toml` の
runner に設定する。これにより:

- アプリ実行中 → reboot が効いて BOOTSEL 化 → load 成功
- 既に BOOTSEL → reboot は静かに失敗 → load が即成功
- 初回(未書き込み)→ 手動 BOOTSEL 後に同じコマンドで書き込める

実装の注意: バッチからの遅延は `ping -n 2 127.0.0.1` を使う。`timeout.exe`
は標準入力がリダイレクトされた環境(cargo runner 配下)で即時異常終了し、
Git Bash 環境では GNU timeout に解決されて引数が非互換になる。

## Consequences

- Windows では `cargo run --release --example demo_rp2040` 一発でボタンレス書き込み。
- Linux/macOS はこの制限がないため、runner を `picotool load -f -x -t elf`
  へ差し替えれば1コマンドで済むはず(未検証、ROADMAP 参照)。
