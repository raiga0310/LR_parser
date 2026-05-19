use eframe::egui;
use lr0_parser_rs::grammar::{Grammar, GrammarError, Symbol, parse_grammar_text, parse_input_text};
use lr0_parser_rs::lr::{CompiledParser, ParserError, compile};
use lr0_parser_rs::runtime::{RuntimeError, run};
use lr0_parser_rs::{StateInfo, StepAction};
use std::fmt;
use super::tree::{draw_tree, layout_ast, tree_pixel_height, H_GAP, NODE_R};

use crate::app::{
    ParseArtifacts, ParseTableAction, ParserApp, ParserKind, ParserStatus,
    SmHighlightView, TraceCursorView,
    build_animation_trace, build_parse_table, terminals_from_grammar,
};
use crate::validation::Validation;

enum UiError {
    Grammar(GrammarError),
    Compile(ParserError),
    Runtime(RuntimeError),
    NotImplemented(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UiError::Grammar(GrammarError::EmptyGrammar) => {
                write!(f, "Grammar is empty.")
            }
            UiError::Grammar(GrammarError::InvalidProductionFormat) => {
                write!(f, "Invalid production format. Use: A -> symbol...")
            }
            UiError::Grammar(GrammarError::MissingLeftHandSide) => {
                write!(f, "Production is missing a left-hand side.")
            }
            UiError::Grammar(GrammarError::NonTerminalTooLong) => {
                write!(f, "Non-terminal must be a single uppercase letter.")
            }
            UiError::Grammar(GrammarError::InvalidSymbol(c)) => {
                write!(f, "Invalid symbol '{c}'. Non-terminals must be uppercase ASCII.")
            }
            UiError::Compile(ParserError::ConflictReducer) => {
                write!(f, "LR conflict: grammar is not LR(0). Check for ambiguous productions.")
            }
            UiError::Compile(ParserError::MissingProduction) => {
                write!(f, "Internal error: production not found during compile.")
            }
            UiError::Runtime(RuntimeError::InvalidAction) => {
                write!(f, "Parse error: unexpected token in input. Check that the input matches the grammar.")
            }
            UiError::Runtime(RuntimeError::EmptyStateStack) => {
                write!(f, "Internal error: state stack is empty.")
            }
            UiError::Runtime(RuntimeError::ExpectedTerminalInput) => {
                write!(f, "Internal error: expected terminal symbol in input.")
            }
            UiError::Runtime(RuntimeError::InvalidReduce) => {
                write!(f, "Internal error: invalid reduce — production not found or stack underflow.")
            }
            UiError::Runtime(RuntimeError::MissingGoto) => {
                write!(f, "Internal error: missing GOTO entry after reduce.")
            }
            UiError::Runtime(RuntimeError::MissingAst) => {
                write!(f, "Internal error: AST stack underflow.")
            }
            UiError::NotImplemented(name) => {
                write!(f, "{name} is not yet implemented.")
            }
        }
    }
}

/// run へ進む前の準備段階（grammar parse, compile, input parse）で起こるエラー型。
/// Grammar/Compile/Input の3バリアントにより、どのフェーズで失敗したかが区別できる。
#[derive(Debug, PartialEq)]
enum ParsePreparationError {
    Grammar(GrammarError),
    Compile(ParserError),
    Input(GrammarError),
}

impl fmt::Display for ParsePreparationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Grammar(e) => write!(f, "Grammar: {}", UiError::Grammar(e.clone())),
            Self::Compile(e) => write!(f, "Compile: {}", UiError::Compile(e.clone())),
            Self::Input(GrammarError::InvalidSymbol(c)) => {
                write!(f, "Input: '{c}' は非終端記号のため入力文字列に使用できません")
            }
            Self::Input(e) => write!(f, "Input: {}", UiError::Grammar(e.clone())),
        }
    }
}

/// grammar parse → compile の依存的逐次チェーンが両方成功したときの中間値。
struct CompiledGrammar {
    grammar: Grammar,
    machine: CompiledParser,
}

/// grammar 側チェーンと input 側チェーンの両方が成功したときのみ生成される実行リクエスト。
struct RunRequest {
    grammar:       Grammar,
    machine:       CompiledParser,
    input_symbols: Vec<Symbol>,
}

/// grammar text の parse → compile を依存的な逐次チェーンとして実行する。
/// parse が失敗すれば compile は行わない。
fn validated_compile(grammar_text: &str) -> Validation<ParsePreparationError, CompiledGrammar> {
    let grammar = match parse_grammar_text(grammar_text) {
        Ok(g)  => g,
        Err(e) => return Validation::invalid(ParsePreparationError::Grammar(e)),
    };
    match compile(&grammar) {
        Ok(machine) => Validation::valid(CompiledGrammar { grammar, machine }),
        Err(e)      => Validation::invalid(ParsePreparationError::Compile(e)),
    }
}

/// input text の parse は grammar と独立して行える。
fn validate_input(input_text: &str) -> Validation<ParsePreparationError, Vec<Symbol>> {
    Validation::from_result(parse_input_text(input_text).map_err(ParsePreparationError::Input))
}

impl ParserApp {
    pub fn show_parser_page(&mut self, ui: &mut egui::Ui) {
        let left_w = ui.available_width() * 0.42;
        let view = self.cursor_view();

        egui::ScrollArea::vertical()
            .id_salt("parser_page_scroll")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    // ── 左列: Input + Parse Table ──────────────────────
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
                        self.show_input_panel(ui, 180.0);
                        ui.add_space(12.0);
                        self.show_parse_table_panel(ui, 300.0, &view);
                    });

                    ui.add_space(12.0);

                    // ── 右列: Trace + AST Formation + State Machine ────
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());
                        self.show_trace_panel(ui, &view);
                        if view.total > 0 {
                            ui.add_space(12.0);
                            self.show_ast_formation_section(ui, &view);
                        }
                        ui.add_space(12.0);
                        self.show_state_machine_panel(ui, &view);
                    });
                });
            });
    }

    fn show_input_panel(&mut self, ui: &mut egui::Ui, text_h: f32) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Algorithm:").size(14.0));
            for kind in [ParserKind::Lr0, ParserKind::Slr, ParserKind::Lalr, ParserKind::Lr1] {
                if ui.selectable_label(self.parser.selected_kind == kind, kind.label()).clicked() {
                    self.parser.selected_kind = kind;
                }
            }
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Production:").size(16.0));
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .id_salt("production_scroll")
            .min_scrolled_height(text_h)
            .max_height(text_h)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.workspace.reducer_string)
                        .desired_width(f32::INFINITY),
                );
            });

        ui.add_space(12.0);
        ui.label(egui::RichText::new("Target String:").size(16.0));
        ui.add_space(5.0);
        ui.text_edit_singleline(&mut self.workspace.input_string);

        ui.add_space(12.0);
        if ui.button(egui::RichText::new("Parse").size(16.0)).clicked() {
            self.handle_parse();
        }

        if !self.parser.result.is_empty() {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(&self.parser.result)
                    .color(egui::Color32::from_rgb(220, 80, 80))
                    .size(13.0),
            );
        }
    }

    fn show_trace_panel(&mut self, ui: &mut egui::Ui, view: &TraceCursorView) {
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Parse Trace:").size(16.0));
        ui.add_space(8.0);

        if view.total == 0 {
            ui.label(
                egui::RichText::new("No trace yet. Parse a grammar and input first.")
                    .color(egui::Color32::GRAY)
                    .size(13.0),
            );
            return;
        }

        let cursor = view.cursor;
        let total = view.total;

        // ── Step counter + nav controls ──────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Step {} / {}", cursor + 1, total))
                    .size(13.0)
                    .strong(),
            );
            ui.add_space(12.0);

            if ui.button("|<").clicked() {
                self.step_to(0);
            }
            if ui.button(" < ").clicked() && cursor > 0 {
                self.step_to(cursor - 1);
            }

            let play_label = if view.is_playing { "⏸" } else { "▶" };
            if ui.button(play_label).clicked() {
                self.toggle_play();
            }

            if ui.button(" > ").clicked() && cursor + 1 < total {
                self.step_to(cursor + 1);
            }
            if ui.button(">|").clicked() {
                self.step_to(total - 1);
            }
        });

        ui.add_space(10.0);

        let Some(step) = &view.step else { return; };

        // ── Action label ──────────────────────────────────────────────────
        let (action_text, action_color) = render_action_label(&step.action);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Action: ").size(14.0).strong());
            ui.label(
                egui::RichText::new(&action_text)
                    .monospace()
                    .size(14.0)
                    .color(action_color),
            );
        });

        ui.add_space(10.0);

        // ── State stack + remaining input ─────────────────────────────────
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("State Stack").size(12.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    for (i, &s) in step.state_stack.iter().enumerate() {
                        let is_top = i + 1 == step.state_stack.len();
                        let color = if is_top {
                            egui::Color32::from_rgb(80, 160, 220)
                        } else {
                            egui::Color32::from_rgb(130, 130, 130)
                        };
                        ui.label(
                            egui::RichText::new(format!("[{}]", s))
                                .monospace()
                                .size(13.0)
                                .color(color),
                        );
                    }
                    if step.state_stack.is_empty() {
                        ui.label(
                            egui::RichText::new("(empty)")
                                .monospace()
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
            });

            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Remaining Input").size(12.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    for (i, &c) in step.remaining_input.iter().enumerate() {
                        let is_head = i == 0;
                        let color = if is_head {
                            egui::Color32::from_rgb(240, 180, 60)
                        } else {
                            egui::Color32::from_rgb(180, 180, 180)
                        };
                        ui.label(
                            egui::RichText::new(c.to_string())
                                .monospace()
                                .size(14.0)
                                .color(color),
                        );
                        ui.add_space(4.0);
                    }
                    if step.remaining_input.is_empty() {
                        ui.label(
                            egui::RichText::new("(empty)")
                                .monospace()
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
            });
        });
    }

    fn show_ast_formation_section(&self, ui: &mut egui::Ui, view: &TraceCursorView) {
        let Some(step) = &view.step else { return; };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("AST Formation:").size(16.0));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "Step {} / {}",
                    view.cursor + 1,
                    view.total
                ))
                .size(13.0)
                .color(egui::Color32::GRAY),
            );
        });
        ui.add_space(5.0);

        egui::ScrollArea::horizontal()
            .id_salt("ast_formation_hscroll")
            .show(ui, |ui| {
                ui.set_min_height(140.0);
                ui.horizontal_top(|ui| {
                    if step.ast_stack.is_empty() {
                        ui.label(
                            egui::RichText::new("(empty)")
                                .monospace()
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                        );
                        return;
                    }

                    let last_idx = step.ast_stack.len() - 1;
                    for (i, node) in step.ast_stack.iter().enumerate() {
                        let is_top = i == last_idx;
                        let stroke_color = if is_top {
                            egui::Color32::from_rgb(240, 200, 60)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        };

                        egui::Frame::new()
                            .stroke(egui::Stroke::new(1.0, stroke_color))
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .corner_radius(4)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("[{}]", i))
                                        .size(11.0)
                                        .color(egui::Color32::GRAY),
                                );
                                ui.add_space(4.0);

                                let layout = layout_ast(node);
                                let tree_w = layout.subtree_width.max(H_GAP);
                                let bottom_y = tree_pixel_height(&layout);
                                let tree_h = bottom_y + NODE_R * 2.0 + 6.0;
                                let size = egui::Vec2::new(tree_w, tree_h);

                                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                                if ui.is_rect_visible(rect) {
                                    draw_tree(
                                        ui.painter(),
                                        rect.min + egui::Vec2::new(0.0, NODE_R),
                                        &layout,
                                    );
                                }
                            });

                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn show_parse_table_panel(&self, ui: &mut egui::Ui, table_h: f32, view: &TraceCursorView) {
        ui.label(egui::RichText::new("Parse Table:").size(16.0));
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Legend:").size(11.0).strong());
            for (label, color, desc) in [
                ("s<n>", egui::Color32::from_rgb(0, 120, 0), "=Shift,"),
                ("r<n>", egui::Color32::from_rgb(0, 0, 120), "=Reduce,"),
                ("g<n>", egui::Color32::from_rgb(120, 60, 0), "=Goto,"),
                ("acc", egui::Color32::from_rgb(120, 0, 120), "=Accept"),
            ] {
                ui.label(
                    egui::RichText::new(label)
                        .color(color)
                        .monospace()
                        .size(11.0),
                );
                ui.label(desc);
            }
        });
        ui.add_space(5.0);

        let highlight = view.step.as_ref().map(|step| (step.from_state, step.lookahead));

        if let ParserStatus::Ready(artifacts) = &self.parser.status {
            self.show_parse_table(ui, &artifacts.symbols, &artifacts.table, highlight, table_h);
        } else {
            ui.label(
                egui::RichText::new("No parse table yet. Parse a grammar first.")
                    .size(13.0)
                    .color(egui::Color32::GRAY),
            );
        }
    }

    fn show_state_machine_panel(&self, ui: &mut egui::Ui, view: &TraceCursorView) {
        ui.label(egui::RichText::new("State Machine:").size(16.0));
        ui.add_space(5.0);

        // ── Trace Debug panel ─────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Trace Debug").size(13.0))
            .id_salt("sm_trace_debug")
            .default_open(false)
            .show(ui, |ui| {
                if let Some(step) = &view.step {
                    // Row 1: cursor + action summary
                    let action_str = match &step.action {
                        StepAction::Shift { terminal, to_state } =>
                            format!("SHIFT '{}' -> {}", terminal, to_state),
                        StepAction::Reduce { rule, pop_count } =>
                            format!("REDUCE {} (pop {})", rule, pop_count),
                        StepAction::Accept =>
                            "ACCEPT".to_string(),
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "cursor={}/{}  action={}",
                            view.cursor + 1, view.total, action_str
                        ))
                        .monospace()
                        .size(11.5),
                    );

                    // Row 2: decision context (pre-action)
                    ui.label(
                        egui::RichText::new(format!(
                            "Decision: from_state={}  lookahead='{}'",
                            step.from_state, step.lookahead
                        ))
                        .monospace()
                        .size(11.5)
                        .color(egui::Color32::from_rgb(160, 200, 255)),
                    );

                    // Row 3: post-action snapshot
                    let stack_str = step.state_stack.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let remaining_str: String = step.remaining_input.iter().collect();
                    let stack_top = step.state_stack.last().map(|s| s.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    ui.label(
                        egui::RichText::new(format!(
                            "Post:  stack_top={}  stack=[{}]  remaining='{}'",
                            stack_top, stack_str, remaining_str
                        ))
                        .monospace()
                        .size(11.5)
                        .color(egui::Color32::from_rgb(200, 255, 180)),
                    );

                    // Row 4: SM edge implication
                    let sm_edge_str = match &step.action {
                        StepAction::Shift { terminal, to_state } =>
                            format!("sm_edge: {} -'{}'-> {}", step.from_state, terminal, to_state),
                        StepAction::Reduce { rule, .. } =>
                            format!("sm_edge: <none — reduce {}; Goto happens next>", rule),
                        StepAction::Accept =>
                            format!("sm_edge: <none — accept at state {}>", step.from_state),
                    };
                    ui.label(
                        egui::RichText::new(sm_edge_str)
                        .monospace()
                        .size(11.5)
                        .color(egui::Color32::from_rgb(255, 220, 130)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("No trace. Parse first.")
                            .size(11.5)
                            .color(egui::Color32::GRAY),
                    );
                }
            });
        ui.add_space(5.0);
        // ─────────────────────────────────────────────────────────────────

        let ParserStatus::Ready(artifacts) = &self.parser.status else {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    egui::RichText::new("No state info yet. Parse a grammar first.")
                        .size(13.0)
                        .color(egui::Color32::GRAY),
                );
            });
            return;
        };

        let sm_highlight = view.sm_highlight();
        let (nodes, total_w, total_h) = layout_sm(&artifacts.state_infos);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            // Horizontal scroll only — vertical size matches content so outer page scroll handles it
            egui::ScrollArea::horizontal()
                .id_salt("sm_graph_scroll")
                .show(ui, |ui| {
                    let canvas = egui::Vec2::new(total_w.max(60.0), total_h.max(60.0));
                    let (rect, _) = ui.allocate_exact_size(canvas, egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        draw_sm(
                            ui.painter(),
                            rect.min,
                            &artifacts.state_infos,
                            &nodes,
                            &sm_highlight,
                            &artifacts.accept_states,
                        );
                    }
                });
        });
    }

    fn show_parse_table(
        &self,
        ui: &mut egui::Ui,
        symbols: &[char],
        table: &[Vec<ParseTableAction>],
        highlight: Option<(usize, char)>,
        height: f32,
    ) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                egui::ScrollArea::both()
                    .id_salt("parse_table_scroll")
                    .max_height(height)
                    .show(ui, |ui| {
                        self.render_table_grid(ui, symbols, table, highlight);
                    });
            },
        );
    }

    fn render_table_grid(
        &self,
        ui: &mut egui::Ui,
        symbols: &[char],
        table: &[Vec<ParseTableAction>],
        highlight: Option<(usize, char)>,
    ) {
        let highlight_col: Option<usize> = highlight.and_then(|(_, sym)| {
            symbols.iter().position(|&s| s == sym)
        });

        egui::Grid::new("parse_table")
            .num_columns(symbols.len() + 1)
            .spacing([25.0, 3.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("State").strong().size(13.0));
                for symbol in symbols {
                    ui.label(egui::RichText::new(symbol.to_string()).strong().size(13.0));
                }
                ui.end_row();

                for (state_id, actions) in table.iter().enumerate() {
                    let is_highlighted_row =
                        highlight.map(|(s, _)| s == state_id).unwrap_or(false);

                    let row_color = if is_highlighted_row {
                        egui::Color32::from_rgb(60, 60, 20)
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    let state_text = egui::RichText::new(state_id.to_string())
                        .monospace()
                        .size(11.0);
                    if is_highlighted_row {
                        ui.label(state_text.color(egui::Color32::from_rgb(240, 220, 80)));
                    } else {
                        ui.label(state_text);
                    }

                    for (col_idx, action) in actions.iter().enumerate() {
                        let action_str = action.as_label();
                        let is_highlight_cell =
                            is_highlighted_row && highlight_col == Some(col_idx);
                        let base_color = match action {
                            ParseTableAction::Shift(_) => egui::Color32::from_rgb(0, 120, 0),
                            ParseTableAction::Reduce(_) => egui::Color32::from_rgb(0, 0, 120),
                            ParseTableAction::Accept => egui::Color32::from_rgb(120, 0, 120),
                            ParseTableAction::Goto(_) => egui::Color32::from_rgb(120, 60, 0),
                            ParseTableAction::Error => egui::Color32::from_rgb(100, 100, 100),
                        };

                        let display_color = if is_highlight_cell {
                            egui::Color32::from_rgb(255, 220, 50)
                        } else {
                            base_color
                        };

                        if action_str.is_empty() {
                            ui.label(
                                egui::RichText::new("-")
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        } else {
                            let text = egui::RichText::new(&action_str)
                                .monospace()
                                .size(11.0)
                                .color(display_color);
                            if is_highlight_cell {
                                egui::Frame::new()
                                    .stroke(egui::Stroke::new(
                                        1.5,
                                        egui::Color32::from_rgb(255, 220, 50),
                                    ))
                                    .inner_margin(egui::Margin::symmetric(3, 1))
                                    .show(ui, |ui| {
                                        ui.label(text);
                                    });
                            } else {
                                ui.label(text);
                            }
                        }
                    }
                    let _ = row_color;
                    ui.end_row();
                }
            });
    }

    fn handle_parse(&mut self) {
        match self.parser.selected_kind {
            ParserKind::Lr0 => self.handle_parse_lr0(),
            other => {
                self.parser.result =
                    UiError::NotImplemented(other.label().to_string()).to_string();
                self.parser.parse_trace.clear();
                self.parser.status = ParserStatus::Empty;
            }
        }
    }

    fn handle_parse_lr0(&mut self) {
        // ── フェーズ1: 独立な2チェーンを Applicative 的に合成 ──────────────────
        let compiled = validated_compile(&self.workspace.reducer_string);
        let input    = validate_input(&self.workspace.input_string);

        let request = match compiled.map2(input, |cg, symbols| RunRequest {
            grammar:       cg.grammar,
            machine:       cg.machine,
            input_symbols: symbols,
        }) {
            Validation::Valid(req) => req,
            Validation::Invalid(errors) => {
                self.parser.result = errors.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.parser.status = ParserStatus::Empty;
                self.parser.parse_trace.clear();
                return;
            }
        };

        // ── フェーズ2: compile 成功後のテーブル構築 ────────────────────────────
        let (symbols, table) = build_parse_table(&request.grammar, &request.machine);
        self.workspace.terminals = terminals_from_grammar(&request.grammar);
        self.apply_default_terminal_types();

        // ── フェーズ3: run（RunRequest への依存的な逐次処理） ──────────────────
        match run(&request.machine, &request.input_symbols).map_err(UiError::Runtime) {
            Ok(_) => {
                self.parser.result.clear();
            }
            Err(e) => {
                self.parser.result = e.to_string();
                self.parser.parse_trace.clear();
                // parse table は表示するが SM は空で返す
                self.parser.status = ParserStatus::Ready(ParseArtifacts {
                    symbols,
                    table,
                    state_infos: vec![],
                    accept_states: vec![],
                });
                return;
            }
        }

        let state_infos = request.machine.state_infos().to_vec();
        let accept_states: Vec<usize> = table.iter().enumerate()
            .filter(|(_, row)| row.iter().any(|a| matches!(a, ParseTableAction::Accept)))
            .map(|(state, _)| state)
            .collect();
        self.parser.status = ParserStatus::Ready(ParseArtifacts {
            symbols,
            table,
            state_infos,
            accept_states,
        });

        self.parser.parse_trace = build_animation_trace(&request.machine, &request.input_symbols).unwrap_or_default();
        self.parser.trace_cursor      = 0;
        self.parser.anim_playing      = false;
        self.parser.anim_last_advance = None;
    }
}

// ── State Machine graph layout & drawing ────────────────────────────────────

const SM_NODE_R: f32 = 22.0;
const SM_H_GAP: f32 = 105.0;
const SM_V_GAP: f32 = 68.0;
const SM_LABEL_OFFSET: f32 = 13.0;
const SM_TOP_PAD: f32 = 100.0; // back-edge ctrl.y goes up to NODE_R + 32 + 36 = 90 px above origin

struct SmNodePos {
    state_id: usize,
    x: f32,
    y: f32,
}

fn layout_sm(state_infos: &[StateInfo]) -> (Vec<SmNodePos>, f32, f32) {
    if state_infos.is_empty() {
        return (vec![], 0.0, 0.0);
    }
    let n = state_infos.len();
    let mut layer = vec![usize::MAX; n];
    let mut queue = std::collections::VecDeque::new();
    layer[0] = 0;
    queue.push_back(0_usize);
    while let Some(s) = queue.pop_front() {
        for (_, next) in &state_infos[s].transitions {
            if *next < n && layer[*next] == usize::MAX {
                layer[*next] = layer[s] + 1;
                queue.push_back(*next);
            }
        }
    }
    let max_reached = layer.iter().filter(|&&l| l != usize::MAX).max().copied().unwrap_or(0);
    for l in layer.iter_mut() {
        if *l == usize::MAX {
            *l = max_reached + 1;
        }
    }
    let num_layers = layer.iter().max().copied().unwrap_or(0) + 1;
    let mut by_layer: Vec<Vec<usize>> = vec![vec![]; num_layers];
    for (s, &l) in layer.iter().enumerate() {
        by_layer[l].push(s);
    }
    let h_margin = SM_NODE_R * 2.0;
    let mut nodes = vec![];
    for (col, states) in by_layer.iter().enumerate() {
        for (row, &s) in states.iter().enumerate() {
            nodes.push(SmNodePos {
                state_id: s,
                x: col as f32 * SM_H_GAP + h_margin,
                y: row as f32 * SM_V_GAP + SM_TOP_PAD,
            });
        }
    }
    // +70 for accept termination arrow (44px arrow + 17px dot + margin)
    let total_w = num_layers as f32 * SM_H_GAP + h_margin * 2.0 + 70.0;
    let max_in_layer = by_layer.iter().map(|v| v.len()).max().unwrap_or(0);
    let total_h = max_in_layer as f32 * SM_V_GAP + SM_TOP_PAD + h_margin;
    (nodes, total_w, total_h)
}

fn sm_bezier_point(p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, t: f32) -> egui::Pos2 {
    let u = 1.0 - t;
    egui::Pos2 {
        x: u * u * p0.x + 2.0 * u * t * p1.x + t * t * p2.x,
        y: u * u * p0.y + 2.0 * u * t * p1.y + t * t * p2.y,
    }
}

fn draw_curve(painter: &egui::Painter, p0: egui::Pos2, ctrl: egui::Pos2, p2: egui::Pos2, stroke: egui::Stroke) {
    let pts: Vec<egui::Pos2> = (0..=12)
        .map(|i| sm_bezier_point(p0, ctrl, p2, i as f32 / 12.0))
        .collect();
    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], stroke);
    }
}

fn draw_arrowhead(painter: &egui::Painter, tip: egui::Pos2, dir: egui::Vec2, color: egui::Color32) {
    if dir.length_sq() < 1e-6 {
        return;
    }
    let perp = egui::Vec2::new(-dir.y, dir.x);
    let base = tip - dir * 11.0;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, base + perp * 5.5, base - perp * 5.5],
        color,
        egui::Stroke::NONE,
    ));
}

fn draw_sm(
    painter: &egui::Painter,
    origin: egui::Pos2,
    state_infos: &[StateInfo],
    nodes: &[SmNodePos],
    highlight: &SmHighlightView,
    accept_states: &[usize],
) {
    let active_edge = highlight.active_edge;
    let source_state = highlight.source_state;
    let result_state = highlight.result_state;

    let max_id = nodes.iter().map(|n| n.state_id).max().unwrap_or(0);
    let mut pos_of: Vec<Option<egui::Pos2>> = vec![None; max_id + 1];
    let mut node_x: Vec<f32> = vec![0.0; max_id + 1];
    for n in nodes {
        let p = origin + egui::Vec2::new(n.x, n.y);
        pos_of[n.state_id] = Some(p);
        node_x[n.state_id] = n.x;
    }

    // ── edges ───────────────────────────────────────────────────────────────
    for info in state_infos {
        let Some(src_pos) = pos_of.get(info.id).and_then(|p| *p) else { continue };
        let src_x = node_x.get(info.id).copied().unwrap_or(0.0);

        // Group transitions by destination
        let mut by_dest: std::collections::BTreeMap<usize, Vec<&Symbol>> = Default::default();
        for (sym, dest) in &info.transitions {
            by_dest.entry(*dest).or_default().push(sym);
        }

        for (dest, syms) in &by_dest {
            let Some(dst_pos) = pos_of.get(*dest).and_then(|p| *p) else { continue };
            let dst_x = node_x.get(*dest).copied().unwrap_or(0.0);

            let is_active = active_edge
                .map(|(from, terminal, to)| {
                    from == info.id
                        && to == *dest
                        && syms.iter().any(|s| matches!(s, Symbol::Terminal(t) if t.0 == terminal))
                })
                .unwrap_or(false);

            let edge_color = if is_active {
                egui::Color32::from_rgb(255, 220, 50)
            } else {
                egui::Color32::from_rgb(130, 130, 150)
            };
            let stroke_w = if is_active { 3.5 } else { 1.5 };
            let stroke = egui::Stroke::new(stroke_w, edge_color);

            let label: String = syms
                .iter()
                .map(|s| match s {
                    Symbol::Terminal(c) => format!("'{}'", c.0),
                    Symbol::NonTerminal(c) => c.0.to_string(),
                })
                .collect::<Vec<_>>()
                .join(",");

            if info.id == *dest {
                // Self-loop
                let loop_c = src_pos - egui::Vec2::new(0.0, SM_NODE_R + 12.0);
                let p0 = src_pos - egui::Vec2::new(SM_NODE_R * 0.5, SM_NODE_R);
                let p2 = src_pos + egui::Vec2::new(SM_NODE_R * 0.5, -SM_NODE_R);
                draw_curve(painter, p0, loop_c, p2, stroke);
                let end_dir = (p2 - loop_c).normalized();
                draw_arrowhead(painter, p2, end_dir, edge_color);
                painter.text(
                    loop_c - egui::Vec2::new(0.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    &label,
                    egui::FontId::monospace(10.5),
                    edge_color,
                );
                continue;
            }

            if src_x < dst_x {
                // Forward edge — straight line
                let dir = (dst_pos - src_pos).normalized();
                let from = src_pos + dir * SM_NODE_R;
                let to = dst_pos - dir * SM_NODE_R;
                painter.line_segment([from, to], stroke);
                draw_arrowhead(painter, to, dir, edge_color);
                let mid = egui::Pos2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
                let perp = egui::Vec2::new(-dir.y, dir.x);
                painter.text(
                    mid + perp * SM_LABEL_OFFSET,
                    egui::Align2::CENTER_CENTER,
                    &label,
                    egui::FontId::monospace(10.5),
                    edge_color,
                );
            } else {
                // Back / same-layer edge — curved above
                let spread = (*dest as f32 * 12.0).min(36.0);
                let from = src_pos - egui::Vec2::new(0.0, SM_NODE_R);
                let to   = dst_pos - egui::Vec2::new(0.0, SM_NODE_R);
                let ctrl = egui::Pos2::new(
                    (from.x + to.x) / 2.0,
                    from.y.min(to.y) - 32.0 - spread,
                );
                draw_curve(painter, from, ctrl, to, stroke);
                let end_dir = (to - ctrl).normalized();
                draw_arrowhead(painter, to, end_dir, edge_color);
                painter.text(
                    ctrl - egui::Vec2::new(0.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    &label,
                    egui::FontId::monospace(10.5),
                    edge_color,
                );
            }
        }
    }

    // ── nodes ────────────────────────────────────────────────────────────────
    for node in nodes {
        let Some(pos) = pos_of.get(node.state_id).and_then(|p| *p) else { continue };
        let is_source = source_state == Some(node.state_id);
        let is_result = result_state == Some(node.state_id);
        let is_accept = accept_states.contains(&node.state_id);
        let (fill, rim) = if is_source {
            // pre-action (decision) node — orange
            (
                egui::Color32::from_rgb(180, 90, 20),
                egui::Color32::from_rgb(255, 200, 50),
            )
        } else if is_result {
            // post-action (destination) node — green
            (
                egui::Color32::from_rgb(45, 90, 70),
                egui::Color32::from_rgb(110, 230, 170),
            )
        } else if is_accept {
            // accept state — violet
            (
                egui::Color32::from_rgb(70, 30, 90),
                egui::Color32::from_rgb(200, 120, 255),
            )
        } else {
            (
                egui::Color32::from_rgb(20, 55, 60),
                egui::Color32::from_rgb(80, 160, 180),
            )
        };
        painter.circle(pos, SM_NODE_R, fill, egui::Stroke::new(1.5, rim));
        // Accept state gets a double ring to stay visible even when animated
        if is_accept {
            painter.circle(
                pos,
                SM_NODE_R + 4.0,
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 120, 255)),
            );
        }
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            &node.state_id.to_string(),
            egui::FontId::monospace(13.0),
            egui::Color32::WHITE,
        );
    }

    // ── Accept termination arrows ────────────────────────────────────────────
    let accept_color = egui::Color32::from_rgb(200, 120, 255);
    let accept_stroke = egui::Stroke::new(2.0, accept_color);
    for &acc in accept_states {
        let Some(pos) = pos_of.get(acc).and_then(|p| *p) else { continue };
        let from = pos + egui::Vec2::new(SM_NODE_R, 0.0);
        let to   = from + egui::Vec2::new(44.0, 0.0);
        painter.line_segment([from, to], accept_stroke);
        draw_arrowhead(painter, to, egui::Vec2::new(1.0, 0.0), accept_color);
        // termination dot
        painter.circle(
            to + egui::Vec2::new(8.0, 0.0),
            6.0,
            accept_color,
            egui::Stroke::NONE,
        );
        painter.circle(
            to + egui::Vec2::new(8.0, 0.0),
            9.0,
            egui::Color32::TRANSPARENT,
            accept_stroke,
        );
        // label
        painter.text(
            from + egui::Vec2::new(22.0, -11.0),
            egui::Align2::CENTER_CENTER,
            "$",
            egui::FontId::monospace(11.0),
            accept_color,
        );
    }
}



// ── Tree layout & drawing ────────────────────────────────────────────────────

fn render_action_label(action: &StepAction) -> (String, egui::Color32) {
    match action {
        StepAction::Shift { terminal, to_state } => (
            format!("SHIFT '{terminal}'  →  State {to_state}"),
            egui::Color32::from_rgb(80, 200, 80),
        ),
        StepAction::Reduce { rule, pop_count } => (
            format!("REDUCE  {rule}  (pop {pop_count})"),
            egui::Color32::from_rgb(100, 140, 255),
        ),
        StepAction::Accept => (
            "ACCEPT".to_string(),
            egui::Color32::from_rgb(200, 100, 220),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::Validation;

    const VALID_GRAMMAR: &str = "E -> E+B\nE -> B\nB -> 0\nB -> 1";
    // E -> EE が Shift/Reduce 競合を引き起こし compile が失敗する文法
    const CONFLICT_GRAMMAR: &str = "E -> <>\nE -> <E>\nE -> EE";

    // ── validated_compile ────────────────────────────────────────────

    #[test]
    fn validated_compile_valid_grammar_returns_valid() {
        assert!(matches!(validated_compile(VALID_GRAMMAR), Validation::Valid(_)));
    }

    #[test]
    fn validated_compile_empty_grammar_returns_grammar_error() {
        assert!(matches!(
            validated_compile(""),
            Validation::Invalid(ref errs) if matches!(errs[0], ParsePreparationError::Grammar(_))
        ));
    }

    #[test]
    fn validated_compile_conflict_grammar_returns_compile_error() {
        assert!(matches!(
            validated_compile(CONFLICT_GRAMMAR),
            Validation::Invalid(ref errs) if matches!(errs[0], ParsePreparationError::Compile(_))
        ));
    }

    // ── validate_input ───────────────────────────────────────────────

    #[test]
    fn validate_input_valid_returns_valid() {
        assert!(matches!(validate_input("1+0"), Validation::Valid(_)));
    }

    #[test]
    fn validate_input_nonterminal_returns_input_error() {
        assert!(matches!(
            validate_input("X"),
            Validation::Invalid(ref errs) if matches!(errs[0], ParsePreparationError::Input(_))
        ));
    }

    // ── 合成フロー（map2） ────────────────────────────────────────────

    #[test]
    fn good_grammar_and_bad_input_yields_single_input_error() {
        let result = validated_compile(VALID_GRAMMAR)
            .map2(validate_input("X"), |_, _| unreachable!());
        assert!(matches!(
            result,
            Validation::Invalid(ref errs)
                if errs.len() == 1 && matches!(errs[0], ParsePreparationError::Input(_))
        ));
    }

    #[test]
    fn compile_fail_and_bad_input_accumulates_both_errors() {
        // 設計価値の中心: compile 失敗と input 失敗が同時に蓄積される
        let result = validated_compile(CONFLICT_GRAMMAR)
            .map2(validate_input("X"), |_, _| unreachable!());
        assert!(matches!(
            result,
            Validation::Invalid(ref errs)
                if errs.len() == 2
                    && matches!(errs[0], ParsePreparationError::Compile(_))
                    && matches!(errs[1], ParsePreparationError::Input(_))
        ));
    }
}
