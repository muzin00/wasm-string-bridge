# wasm-string-bridge

WebAssembly Component Model を使用し、ホストとゲスト間で型安全に文字列をやり取りする最小動作環境。

## 構成

| コンポーネント | 役割 | ディレクトリ |
| --- | --- | --- |
| WIT | インターフェース定義（Single Source of Truth） | `wit/` |
| Host | `wasmtime` で `.wasm` をロード・実行 | `host/` |
| Guest (Rust) | `process-string` を実装する Wasm コンポーネント（`wit-bindgen` のマクロでバインディングを生成） | `guests/rust/` |
| Guest (JS) | `process-string` を実装する Wasm コンポーネント（ComponentizeJS / `jco componentize` でビルド） | `guests/js/` |

## セットアップ

```sh
# Rust ゲスト用
rustup target add wasm32-wasip2

# JS ゲスト用（Node.js が必要。`.nvmrc` で指定したバージョンを推奨）
cd guests/js && npm install
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
    ├── rust/               # Rust 製 Wasm ゲストコンポーネント
    └── js/                 # JavaScript 製 Wasm ゲストコンポーネント (ComponentizeJS)
```

## 実行手順

### Rust ゲスト

```sh
# 1. Guest を Wasm Component としてビルド
cargo build -p guest-rust --target wasm32-wasip2 --release

# 2. Host から Guest を呼び出す（--guest 省略時のデフォルト）
cargo run -p host -- "rust wasm"
# => RUST WASM

# もしくは明示的に指定
cargo run -p host -- --guest=rust "rust wasm"
```

### JS ゲスト

```sh
# 1. Guest を Wasm Component としてビルド（dist/guest_js.wasm が生成される）
cd guests/js
npm install        # 初回のみ
npm run build
cd ../..

# 2. Host から JS ゲストを呼び出す
cargo run -p host -- --guest=js "rust wasm"
# => RUST WASM
```

Guest 側の `process-string` が `unimplemented!()` のままだと実行時に
trap して Host は `Err` を返すが、Host 側のコード経路（コンポーネントの
ロード〜呼び出し）は完走する。
