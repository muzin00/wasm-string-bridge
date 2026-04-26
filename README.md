# wasm-string-bridge

WebAssembly Component Model を使用し、ホストとゲスト間で型安全に文字列をやり取りする最小動作環境。

## 構成

| コンポーネント | 役割 | ディレクトリ |
| --- | --- | --- |
| WIT | インターフェース定義（Single Source of Truth） | `wit/` |
| Host | `wasmtime` で `.wasm` をロード・実行 | `host/` |
| Guest (Rust) | `process-string` を実装する Wasm コンポーネント（`wit-bindgen` のマクロでバインディングを生成） | `guests/rust/` |

## セットアップ

```sh
rustup target add wasm32-wasip2
```

## ディレクトリ構成

```
wasm-string-bridge/
├── Cargo.toml              # workspace 定義
├── rust-toolchain.toml     # stable + wasm32-wasip2
├── wit/
│   └── interface.wit       # WIT (Single Source of Truth)
├── host/                   # wasmtime ホストアプリケーション
└── guests/
    └── rust/               # Rust 製 Wasm ゲストコンポーネント
```
