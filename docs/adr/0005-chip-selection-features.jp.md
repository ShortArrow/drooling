# ADR 0005: チップ (RP2040 / RP2350) は相互排他な cargo feature で選択する

## Status

Accepted (2026-08-12)

## Context

reset interface のうちチップに依存するのはリセットの実行だけで、
RP2040 は boot ROM の `reset_to_usb_boot` + watchdog レジスタ、RP2350 は
ROM の `reboot` API(`rp235x_hal::reboot`)を使う。プロトコル層
(`protocol`、`ms_os_20`)と UsbClass の記述子処理はチップ非依存。
単一クレートで両チップに対応したい。

制約: `rp235x-pac` 0.2 は `target_arch = "arm"` 全体で cortex-m 用
ソースを選択するのに、cortex-m 系依存の宣言は thumbv8m ターゲット限定の
ため、thumbv6m(RP2040)ビルドに混入するとコンパイル不能になる。

## Decision

- **features**: `default = []`、`rp2040 = ["dep:rp2040-hal"]`、
  `rp2350 = ["dep:rp235x-hal"]`。同時有効は `compile_error!` で拒否する。
  `default = []` で常に明示選択とする。理由: 両チップの対称性、
  `default-features = false` という間接的な指定の廃止、クレートが
  チップを推測しないこと。チップ未指定の `compile_error!` は arm/none
  ターゲット限定にして、ホスト(テスト・docs.rs・publish 検証)は
  チップなしでビルド可能に保つ。
- **チップ差は `picotool_reset.rs` の私有関数2つ
  (`enter_bootsel` / `reboot_to_application`)の cfg 分岐に閉じ込める。**
  RP2350 の `disable_interface_mask` は bit0 = MSD 無効、bit1 = picoboot
  無効にマップする。RP2350 の ROM API には activity pin 引数が無いため、
  プロトコル上は受理して無視する。
- **例とリンカレイアウトもチップ別**: `[[example]]` の
  `required-features`、`memory/rp2040/` と `memory/rp2350/` の分離。
  `.cargo/config.toml` のターゲット選別は cfg() 述語
  (`target_abi = "eabi"` / `"eabihf"`)で行う — cargo はドットを含む
  トリプル名の `[target.'thumbv8m.main-none-eabihf']` セクションを
  無視するため(1.97.1 で実証)。
- **rp235x 系の例依存はターゲットスコープの dev-dependencies に置く**
  (上記 rp235x-pac 制約を thumbv6m ビルドから隔離する)。

## Consequences

- 利用側は `drooling = { version = "...", features = ["rp2040"] }` または
  `features = ["rp2350"]` と書く。RP2350 行から `default-features = false`
  が消え、両チップの記述が対称になる。
- コマンドラインでも `--no-default-features` は不要になり、
  `--features rp2040` / `--features rp2350` だけで足りる。
- RP2350 は Windows でも単発 `picotool load -f` が使える(2段階の
  `tools/flash.cmd` が必須なのは RP2040 + Windows のみ)。
- 既存の `drooling = "0.1"`(チップ未指定)はファームターゲットで
  `compile_error!` になるため、0.2.0 の Breaking として出荷する。
  ホストビルド(テスト・docs.rs・`cargo publish` の検証ビルド)は
  チップなしのまま通る。
