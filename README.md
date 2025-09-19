# LR Parser implemented in Rust
LR構文解析(LR(0))法のRust実装

## Uasage(使い方)
`reducer`が生成規則に対応しています

```sh
cargo run
```

## 前提
- rustが実行できる環境

## 生成規則の書き方

### 生成規則
- `->`で左辺右辺を識別しています
- 右辺に空白記号がある場合カットされます
