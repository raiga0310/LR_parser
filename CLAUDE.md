# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
# Build
cargo build

# Run GUI application
cargo run

# Run all tests
cargo test

# Run a single test by name
cargo test <test_name>

# Run tests in a specific module
cargo test grammar::tests
cargo test runtime::tests
```

## Architecture

This project is a dual-crate workspace in a single `Cargo.toml`:

- **Library** (`lr0_parser_rs`, `src/lib.rs`): pure parsing logic — no GUI, no I/O
- **Binary** (`lr0-parser-gui`, `src/main.rs`): egui GUI that wraps the library

### Library pipeline

```
grammar::parse_grammar_text(text)  →  Grammar
lr::compile(&grammar)              →  CompiledParser
runtime::run(&machine, &symbols)   →  ParserResult { ast: AstNode }
runtime::build_trace(...)          →  Vec<ParseStep>   (step-by-step for animation)
```

Each stage is in its own module:

| Module | Responsibility |
|---|---|
| `grammar.rs` | Text → `Grammar` (productions, terminals, non-terminals). Non-terminals must be a single uppercase ASCII char; `$` is the implicit EOF terminal. |
| `lr.rs` | `Grammar` → `CompiledParser` (LR(0) item sets, action/goto tables). `compile()` returns `Err(ParserError::ConflictReducer)` on Shift/Reduce or Reduce/Reduce conflicts — conflicts are hard errors, never silently resolved. |
| `runtime.rs` | `CompiledParser` + input → `ParserResult`. Also exports `build_trace` which returns `Vec<ParseStep>` capturing the full step history for the animation UI. |
| `ast.rs` | `AstNode` enum: `Terminal(char)` or `NonTerminal(char, Vec<AstNode>)`. |

### GUI binary structure

```
main.rs                  — window setup (1200×800 initial, 800×600 min)
app.rs                   — ParserApp state struct + eframe App impl
  ParserApp              — all UI state (parse trace, animation cursor, generator AST, …)
  ParserKind enum        — Lr0 | Slr | Lalr | Lr1 (only Lr0 is implemented)
  build_parse_table()    — converts CompiledParser → 2D ParseTableAction grid for display
  build_animation_trace()— thin wrapper around runtime::build_trace
pages/
  parser.rs              — Parser tab UI: input panel, animation trace panel,
                           AST formation section, parse table section
  generator.rs           — Generator tab UI: terminal role selectors,
                           code preview cards (AST tree, source, eval, code, run)
  tree.rs                — Shared egui Painter tree renderer used by both pages
                           (LayoutNode, layout_ast, draw_tree, tree_pixel_height)
generator_engine.rs      — GeneratorEngine: AST → Rust source code generation.
                           run_generated_code() writes a temp file and shells out to rustc.
```

### Key design rules

- **LR conflicts are errors**: `lr::compile` returns `Err` on any conflict. The UI surfaces this as a human-readable message via `UiError::Compile`.
- **Functional core / imperative shell**: library modules (`grammar`, `lr`, `runtime`) are pure functions. Side effects (file I/O, process spawning) are confined to `generator_engine::run_generated_code`.
- **Algorithm stubs**: `ParserKind::Slr / Lalr / Lr1` exist in `app.rs` as extension points. Selecting them shows "not yet implemented" via `UiError::NotImplemented`. Actual implementations are left for future work.
- **Shared tree renderer**: `pages/tree.rs` is `pub(super)` — visible only within the `pages` module. Both `parser.rs` and `generator.rs` import from it via `use super::tree::*`.

### Grammar format

One production per line, `LHS -> RHS`:
- LHS: exactly one uppercase ASCII letter
- RHS: sequence of uppercase letters (non-terminals) and any other character (terminals)
- `$` is reserved as the EOF terminal (automatically appended to input)
- Whitespace in RHS is stripped

Sample grammars are in the repo root: `reducer` (arithmetic) and `paren_reducer` (non-LR(0), used in tests).
