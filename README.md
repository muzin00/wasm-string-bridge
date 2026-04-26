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

## 実行手順

```sh
# 1. Guest を Wasm Component としてビルド
cargo build -p guest-rust --target wasm32-wasip2 --release

# 2. Host から Guest を呼び出す（第 1 引数が Guest への入力）
cargo run -p host -- "rust wasm"
# => RUST WASM   (Guest の process-string が本実装されている場合)
```

Guest 側の `process-string` が `unimplemented!()` のままだと実行時に
trap して Host は `Err` を返すが、Host 側のコード経路（コンポーネントの
ロード〜呼び出し）は完走する。
