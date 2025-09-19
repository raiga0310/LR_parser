# LR Parser implemented in Rust
LR構文解析(LR(0))法のRust実装

## 概要
このプロジェクトは、LR(0)構文解析器をRustで実装し、GUI付きのパーサー・ジェネレーターアプリケーションを提供します。

## 機能
- **LR(0)構文解析**: 文法規則に基づくパース処理
- **AST生成**: 抽象構文木の自動生成
- **GUIインターフェース**: 直感的な操作が可能
- **コードジェネレーター**: ASTから実行可能なRustコードを生成
- **終端記号の型設定**: 各終端記号に対する型の選択機能

## 使い方

### 基本的な実行
GUIアプリケーションを起動:
```sh
cargo run --bin lr0-parser-gui
```

コマンドライン版の実行:
```sh
cargo run
```

### GUIの使用方法

#### 1. Parserページ
- **Production**: 生成規則を入力（デフォルト: 数式文法）
- **Target String**: パースしたい文字列を入力
- **Parse**: パース処理を実行してASTを生成

#### 2. Generatorページ
- **Terminal Symbols**: 終端記号の一覧と型選択
- **Generate Code**: ASTから実行可能なRustコードを生成
- **Generate Result**: 生成されたコードの表示

## 生成規則の書き方

### 文法の記述
- `->`で左辺と右辺を区切る
- 右辺の空白文字は自動的に除去される
- 非終端記号: 大文字（A, B, E など）
- 終端記号: 小文字・数字・記号（+, *, 0, 1 など）

### デフォルト文法の例
```
E -> E*B
E -> E+B
E -> B
B -> 0
B -> 1
```

この文法は以下を表現:
- `E`: 式（Expression）
- `B`: 基本項（Basic term）
- `*`: 乗算演算子
- `+`: 加算演算子
- `0`, `1`: 数値リテラル

## 終端記号の型

Generatorページで各終端記号に以下の型を設定可能:
- **Add**: 加算演算子として扱う
- **Mul**: 乗算演算子として扱う
- **L_paren**: 左括弧として扱う
- **R_paren**: 右括弧として扱う
- **Num**: 数値リテラルとして扱う

## コード生成例

### 入力
- **Production**: `E -> E+B\nE -> B\nB -> 0\nB -> 1`
- **Target String**: `1+0`
- **終端記号の型**: `+`→Add, `1`→Num, `0`→Num

### 生成されるコード
```rust
// Generated Rust code from AST
fn main() {
    let result_1 = (1 + 0); // Evaluated expression
    println!("Result: {}", result_1);
}
```

## プロジェクト構成

```
src/
├── main.rs          # エントリーポイント
├── lib.rs           # LR(0)パーサーのコア実装
├── app.rs           # GUIアプリケーションのメイン構造体
└── pages/
    ├── mod.rs       # ページモジュールの宣言
    ├── parser.rs    # Parserページの実装
    └── generator.rs # Generatorページの実装
```

## 依存関係

### 必須依存
- `eframe`: GUIフレームワーク（egui）

### 開発要件
- Rust 1.70以上
- Cargo

## インストールと実行

### 1. リポジトリのクローン
```sh
git clone https://github.com/raiga0310/LR_parser.git
cd LR_parser
```

### 2. 依存関係のインストール
```sh
cargo build
```

### 3. 実行
```sh
# GUI版
cargo run --bin lr0-parser-gui

# CLI版
cargo run
```

## テスト
```sh
cargo test
```

## プロジェクト構造
```
src/
├── main.rs                # エントリーポイント
├── lib.rs                 # ライブラリルート
├── app.rs                 # メインアプリケーション構造体
├── generator_engine.rs    # コード生成エンジン
└── pages/                 # UIページモジュール
    ├── mod.rs            # ページモジュール管理
    ├── parser.rs         # パーサーページUI
    └── generator.rs      # ジェネレーターページUI
```

### アーキテクチャの責務分離
- **app.rs**: GUI状態管理とページ切り替え
- **generator_engine.rs**: AST解析、コード生成、実行処理
- **pages/parser.rs**: パーサーページのUI処理
- **pages/generator.rs**: ジェネレーターページのUI処理
