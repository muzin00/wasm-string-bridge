# wasmtime の 4 役: Engine / Component / Linker / Store

wasmtime で Wasm Component を動かすときに必ず登場する 4 つのオブジェクト。
それぞれの責務を抽象化して整理する。

## 全体像

```
Engine      ← コンパイラ・設定（共有・長命・不変）       … 工場
  └─ Component  ← コンパイル済み + 型情報を持った設計図   … 鋳型
       │
   ╔══════════════════════════════════════════════╗
   ║  ここから "実行ごとの世界" が始まる              ║
   ╠══════════════════════════════════════════════╣
   ║  Linker  ← 依存解決テーブル（共有可）           ║
   ║  Store   ← 全状態の箱（専有・可変・短命）★     ║
   ║       └─ Instance ← Store 内に住む Wasm 実体  ║
   ╚══════════════════════════════════════════════╝
```

**Store より上は "準備"、Store からが "実行"**。

## それぞれの抽象化

### Engine — 「コンパイル環境」

- Wasm を実行可能なネイティブコードに変換するためのコンパイラ環境
- JIT/AOT 設定、コンパイラ (Cranelift / Winch)、コードキャッシュ、メモリ allocator を保持
- **重いが共有可能**。アプリ起動時に 1 個作って使い回す
- Express / FastAPI の `app` インスタンスのライフサイクルに近いが、責務はもっと狭く「コンパイラ層」のみ

### Component — 「型付きの設計図」

- Wasm バイナリを **読み・検証・コンパイル** した不変表現
- **OOP のクラス定義**、**Docker のイメージ**、**JIT 済みの `.class`** に相当
- WIT 由来の **型情報を保持**しているのが古典 Wasm `Module` との決定的な違い
  → bindgen! が `&str` ↔ `String` の型安全な API を生成できる根拠
- **不変・共有可能・状態を持たない**。複数 Store で同時インスタンス化できる
- **Shared-nothing**: Component インスタンス間で linear memory・table を一切共有しない
  → 異言語の Component を再コンパイル無しで合成できる Component Model のキラー設計

### Linker — 「依存解決テーブル ≒ DI コンテナ」

- Guest が宣言した import に対し、Host 側の実装を結びつける
- 動的リンカ (`ld.so`) が名前の由来。「未解決シンボル」を埋める役
- `Linker<T>` の `T` は **Store の state の型**
  → Host 関数が呼ばれたときに Store のどんな状態にアクセスするかを型で表現
- 用途:
  - WASI 実装の登録 (`wasmtime_wasi::add_to_linker_sync`)
  - 自分で書いた Host 関数の登録 (`linker.func_wrap`)
  - 別 Component の export を import に繋ぐ合成

### Store — 「全状態の箱 ≒ プロセス」

- 1 回の Wasm 実行に必要な **全可変状態**を抱える容れ物
- 中身:
  - linear memory / table / globals
  - Host の state (`T`、今回なら `State { ctx, table }`)
  - WASI のリソーステーブル
  - fuel / epoch カウンタ、trap 状態
- **専有・可変・短命** (Engine/Component/Linker と対照的)
- ライフサイクルは「作る → instantiate → call → drop」で全状態が一括解放
- 比喩としては **OS のプロセス** が一番しっくりくる
  - linear memory ≒ 仮想メモリ
  - State ≒ 環境変数・FD テーブル
  - drop ≒ プロセス終了

## なぜ 4 つに分かれているのか

| | 共有可 | 不変 | ライフタイム |
| --- | --- | --- | --- |
| Engine | ✅ | ✅ | 長命 |
| Component | ✅ | ✅ | 長命 |
| Linker | ✅ | ✅ | 長命 |
| **Store** | ❌ | ❌ | **短命・使い捨て** |

Store だけが「**専有・可変・短命**」という性質を持つ。これにより:

- shared-nothing が物理的に強制される
- 状態リーク・クロスコンタミを防ぐ
- GC やメモリ管理を Store 単位で完結

「リクエストごとに新しい Store を作って捨てる」が FaaS / プラグイン基盤の典型パターン。

## WASI を組み込むときの追加要素

```rust
struct State {
    ctx: WasiCtx,         // 「何ができるか」のルールブック (静的)
    table: ResourceTable, // 「今何を持っているか」の所持品リスト (動的)
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
```

- `WasiCtx` = 設定 (preopened dirs, inherit_stdio など、起動時に決める)
- `ResourceTable` = 実体 (開いたファイルハンドル等、実行中に増減)
- `WasiView` = 「Store の State から WasiCtx と ResourceTable を取り出す方法」を WASI 実装に教えるトレイト
- `ResourceTable` は WASI 専用ではなく、自分で WIT に `resource` 型を定義したときの汎用部品でもある

## 今回ハマったポイント

**WIT 上は import を書いていなくても、`wasm32-wasip2` ターゲットの std が WASI を要求する**ため、Host 側で `wasmtime-wasi` を Linker に登録する必要があった。

```
unimplemented!() → panic → std がエラーを stderr に書く
                → wasi:io/poll や wasi:cli/stderr を import
                → Linker に WASI 実装が登録されていないと instantiate 失敗
```

ネイティブの `println!` が裏で `write(2)` syscall を呼ぶのと全く同じ構図。
**std の I/O は OS あるいは WASI 経由で必ず外部に出る**。
