# プログラミング言語コミュニティの "合流" としての Wasm

Wasm Component Model の真の革新は技術仕様にあるのではなく、
**過去の言語横断の試みが届かなかった "言語コミュニティの対等な協調" を実現しつつある**点にある。

## 中心思想: 「言語を統合せず、境界の ABI だけ揃える」

過去の "言語横断" は常に **言語側に妥協を強いる** 形だった:

| アプローチ | 妥協の中身 |
| --- | --- |
| **JVM** | Scala/Kotlin/Clojure は JVM の型システム・GC モデルを受け入れる |
| **.NET CLR** | F#/VB.NET は CLR の制約に従う |
| **GraalVM Truffle** | 各言語を Truffle のインタプリタ実装に書き直す必要 |
| **CORBA / SOAP** | IDL や XML の重さで普及せず |
| **LLVM** | コンパイル時のメモリモデルを LLVM に揃える必要 |

これらは全部「**共通基盤を作るから、言語側がそれに合わせろ**」という統合戦略。
結果として言語の特性が失われたり、エコシステムが二級市民化したりした。

Component Model はこれと真逆のアプローチを取る:

> **「どんな言語も、自分のランタイムを丸ごと持ち込んで Wasm にしていい。境界でだけ ABI を合わせろ」**

```
従来 (JVM 等):
  Java / Scala / Kotlin → JVM の制約に揃える → 1 つの VM
                          (言語側が妥協する)

Component Model:
  Rust       → Rust ランタイム同梱 ┐
  JS         → SpiderMonkey 同梱   ├→ 各々独立した Wasm Component
  Python     → CPython 同梱        │   (境界でだけ Canonical ABI)
  Go         → Go ランタイム同梱   ┘
              (言語側は妥協しない)
```

## 技術的な土台: Shared-nothing

これを可能にしているのが Component Model の **shared-nothing** 設計:

```
[ JVM 型 ]
全言語が同じヒープを共有 → GC モデル・型表現の統一が必要 → 言語の特性が制限される

[ Component Model ]
各 Component が独立した linear memory → GC モデル・型表現は各々自由
境界でだけ "値を Canonical ABI でコピー" すれば良い → 言語は自分の流儀を保てる
```

「**メモリを共有しない**」という一見不便な制約が、「**言語を統合しない**」を実現するための代償になっている。

## ネットワークプロトコルの哲学を、プロセス内に持ち込んだ

| | マイクロサービス | Component Model |
| --- | --- | --- |
| 境界 | ネットワーク | プロセス内 (Wasm の linear memory 境界) |
| 通信コスト | ms 単位 | μs 単位 |
| 隔離強度 | 強 (別プロセス・別マシン) | 中 (同プロセスの Wasm Sandbox) |
| 型契約 | OpenAPI / Protobuf | WIT |
| シリアライズ | JSON / Protobuf bytes | Canonical ABI |

つまり Component Model は **「マイクロサービスの境界をプロセス内まで持ち込んだ」** 設計。
gRPC が「言語自由・型契約だけ揃える」を実現したのと同じ哲学を、ネットワークコストなしで実現した。

## 動的言語の Wasm 化パターン

JS, Python のような動的言語は **言語ランタイムごと Wasm 化して同梱** する:

| ツール | 同梱されるランタイム |
| --- | --- |
| componentize-js (jco) | **SpiderMonkey** (Wasm 化) |
| componentize-py | **CPython** (Wasm 化) |
| TinyGo wasi-p2 | TinyGo ランタイム |

これは PyInstaller の Wasm 版と捉えるのが正確。jco の "本体" は事実上 SpiderMonkey。
ユーザの JS は機械語にコンパイルされず、テキストのまま埋め込まれて実行時に SpiderMonkey が解釈する。

二段構造:
```
wasmtime (Wasm ランタイム, Rust 製)
   ↓ Wasm 命令を実行
SpiderMonkey (Wasm として配布)
   ↓ JS テキストを解釈
あなたの JavaScript
```

注意: componentize-js の SpiderMonkey は **JIT 無効のインタプリタモード**。
ネイティブ SpiderMonkey の多段 JIT (Baseline/Ion/Warp) は使えない。
これが JS Guest がネイティブより遅い理由。

## 各言語が持ち寄っているもの

各コミュニティが **自分の得意を持ち寄っている** のが健全:

| コミュニティ | 持ち寄っているもの |
| --- | --- |
| Rust | wasmtime 本体、ツールチェイン (wasm-tools, wit-bindgen) |
| Mozilla | SpiderMonkey、長年の言語実装ノウハウ |
| Python | componentize-py、CPython の Wasm 対応 |
| Go (TinyGo) | 軽量 Go ランタイム、組込み向け最適化 |
| C/C++ | wasi-libc、wasi-sdk |
| JavaScript | jco、Cloudflare/Deno の実運用フィードバック |
| Java | Chicory、エンタープライズ要求事項 |

奪い合いではなく、**寄付し合っている** 関係 — Linux カーネル開発に近い健全な OSS の協調。

## 中立性の担保

過去の言語横断の試みが失敗した最大の理由は **中立性の欠如**:
- JVM = Sun (現 Oracle) 主導
- .NET = Microsoft 主導
- GraalVM = Oracle Labs 主導

Component Model は意識的に中立な構造になっている:
- **W3C WebAssembly Working Group** (公式標準化)
- **Bytecode Alliance** (実装の共同所有: Mozilla, Fastly, Microsoft, Intel, Google ほか)
- **WASI Subgroup** (システム API の中立な策定)

WIT 仕様の議論で「Rust の `Result<T, E>` か Python の例外か JS の Promise か」を、
特定言語に倒さず `result<T, E>` という抽象に着地できたのは、この対等性ゆえ。

## 何が "美しい" のか

| 感じる美しさ | 背景 |
| --- | --- |
| 対等性 | どの言語も二級市民にならない |
| 多様性の保持 | 統合せず、それぞれの個性を残したまま協調 |
| 協調 | ゼロサムではなく、互いに価値を持ち寄る |
| オープンさ | 中立な標準化、特定企業に支配されない |
| 歴史的意義 | 過去の試みが届かなかった所に届きそうな期待感 |

これは **健全な技術エコシステムの理想形**そのもの。
ソフトウェア工学の長年の理想 (high cohesion / low coupling / information hiding / composability)
を **言語横断の粒度で初めて実現** しつつある。

## 一言まとめ

> **「同じ場所で動かすけど、互いに干渉しない」「協調するけど、依存しない」「合成するけど、変質させない」**

各言語コミュニティが **統合されて消える** のではなく **合流して並び立つ**。
これが Component Model の哲学的中核であり、エコシステムの未来形。
