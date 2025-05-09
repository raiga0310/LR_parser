# LR Parser implemented in Rust
LR構文解析(LR(0))法のRust実装

## Uasage(使い方)
`group.csv`が構文解析表、`reducer`が生成規則に対応しています

```sh
cargo run
```

## 前提
- rustが実行できる環境
- 構文解析表が正しい場合のみ正常に動作します(構文解析表が間違っていた場合の動作は一切保障しません、Contributionは大歓迎です！)

## 構文解析表と生成規則の書き方
※LR構文解析について理解している方むけの説明になります
どちらも最後にLFかCRLFがあるとパニックします

### 構文解析表
- `S<usize>`で`Shift(q_<usize>)`
- `R<usize>`で`Reduce(R_<usize>)`
- `A`で`Accept`
- `G<usize>`で`Goto <usize>`
- それ以外はエラー(CSVの処理実装の都合とみやすさのため便宜的に`e`を入れています)

### 生成規則
- `->`で左辺右辺を識別しています
- 右辺に空白記号がある場合カットされます

## `group.csv`について
参照: [LR法](https://wikipedia.org/wiki/LR_parser)
