# Pico SDK との関係

[English](PICO-SDK.md) | [日本語](PICO-SDK.jp.md)

このクレートがやっていることは、すべて C の世界に先例があり、公式の
Pico SDK が提供している。このページは2つの世界の対応関係を示す。

## SDK が C/C++ でやっていること

SDK のプロジェクトでは、reset interface は `pico_stdio_usb`
コンポーネントの一部。利用者が書くのは実質2行:

```cmake
pico_enable_stdio_usb(my_target 1)   # CMakeLists.txt
```

```c
stdio_init_all();                     // main() の冒頭
```

これだけで SDK が TinyUSB ごと複合デバイス一式 — stdio 用の CDC
シリアル、vendor reset interface、Windows に WinUSB をバインドさせる
Microsoft OS 2.0 ディスクリプタ — を組み込む。利用者は USB コードを
1行も書かない。reset interface は USB stdio が有効なら既定で有効
(`PICO_STDIO_USB_ENABLE_RESET_VIA_VENDOR_INTERFACE`)。

ただしこの自動化には条件がある: USB stdio のおまけとして付いてくる
構造なので、`stdio_usb` を使わず TinyUSB を自前で構成する C プログラム
では自動では入らず、自分で組み込むことになる。

## drooling は何か

`drooling` は同じ機能の Rust `usb-device` エコシステム版。動作する
対応物が無かったので、このクレートで用意した。ワイヤ上は SDK 実装と完全に同一 —
同じ class request、同じディスクリプタ、同じ device interface GUID —
なので、ホストツールから drooling 製ファームと SDK 製ファームは
区別できない。

形の上での意図的な違いが1つ: SDK では reset interface は stdio の
おまけだが、こちらは**独立した USB クラス**(`PicotoolReset`)として
提供し、任意の複合デバイスに組み込める。デモのように CDC シリアルと
組み合わせるのは選択であって必須ではない。

## 対応表

| | C の世界 | Rust の世界 |
|---|---|---|
| ファームウェア側 | Pico SDK(`pico_stdio_usb`) | `drooling` |
| ホスト側 | picotool | `drool`(または picotool — プロトコルは共通) |
