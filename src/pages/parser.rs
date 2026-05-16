use eframe::egui;
use lr0_parser_rs::grammar::{Grammar, GrammarError, Symbol, parse_grammar_text, parse_input_text};
use lr0_parser_rs::lr::{CompiledParser, ParserError, compile};
use lr0_parser_rs::runtime::{RuntimeError, run};
use lr0_parser_rs::StepAction;
use std::fmt;
use super::tree::{draw_tree, layout_ast, tree_pixel_height, H_GAP, NODE_R};

use crate::app::{
    ParseTableAction, ParserApp, ParserKind, build_animation_trace, build_parse_table,
    terminals_from_grammar,
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

/// grammar 側のエラー（parse or compile）と input 側のエラーを区別するUI層エラー型。
/// これにより、両チェーンが互いに独立して失敗したとき両方のエラーを蓄積できる。
#[derive(Debug)]
enum InputValidationError {
    Grammar(GrammarError),
    Compile(ParserError),
    Input(GrammarError),
}

impl fmt::Display for InputValidationError {
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
fn validated_compile(grammar_text: &str) -> Validation<InputValidationError, CompiledGrammar> {
    let grammar = match parse_grammar_text(grammar_text) {
        Ok(g)  => g,
        Err(e) => return Validation::invalid(InputValidationError::Grammar(e)),
    };
    match compile(&grammar) {
        Ok(machine) => Validation::valid(CompiledGrammar { grammar, machine }),
        Err(e)      => Validation::invalid(InputValidationError::Compile(e)),
    }
}

/// input text の parse は grammar と独立して行える。
fn validate_input(input_text: &str) -> Validation<InputValidationError, Vec<Symbol>> {
    Validation::from_result(parse_input_text(input_text).map_err(InputValidationError::Input))
}

impl ParserApp {
    pub fn show_parser_page(&mut self, ui: &mut egui::Ui) {
        let avail_h = ui.available_height();
        let top_row_h = (avail_h * 0.40).max(220.0);
        let ast_row_h = (avail_h * 0.22).max(120.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.show_input_panel(ui, top_row_h);
                ui.add_space(15.0);
                self.show_trace_panel(ui, top_row_h);
            });

            ui.add_space(8.0);
            self.show_ast_formation_section(ui, ast_row_h);

            ui.add_space(8.0);
            self.show_parse_table_section(ui);
        });
    }

    fn show_input_panel(&mut self, ui: &mut egui::Ui, panel_h: f32) {
        let text_h = (panel_h * 0.45).max(80.0);
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width() * 0.35, panel_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Algorithm:").size(14.0));
                    for kind in [ParserKind::Lr0, ParserKind::Slr, ParserKind::Lalr, ParserKind::Lr1] {
                        if ui.selectable_label(self.selected_kind == kind, kind.label()).clicked() {
                            self.selected_kind = kind;
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
                        ui.text_edit_multiline(&mut self.reducer_string);
                    });

                ui.add_space(15.0);
                ui.label(egui::RichText::new("Target String:").size(16.0));
                ui.add_space(5.0);
                ui.text_edit_singleline(&mut self.input_string);

                ui.add_space(15.0);
                if ui.button(egui::RichText::new("Parse").size(16.0)).clicked() {
                    self.handle_parse();
                }

                if !self.parser_result.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&self.parser_result)
                            .color(egui::Color32::from_rgb(220, 80, 80))
                            .size(13.0),
                    );
                }
            },
        );
    }

    fn show_trace_panel(&mut self, ui: &mut egui::Ui, panel_h: f32) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), panel_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Parse Trace:").size(16.0));
                ui.add_space(8.0);

                if self.parse_trace.is_empty() {
                    ui.label(
                        egui::RichText::new("No trace yet. Parse a grammar and input first.")
                            .color(egui::Color32::GRAY)
                            .size(13.0),
                    );
                    return;
                }

                let total = self.parse_trace.len();
                let cursor = self.trace_cursor;

                // ── Step counter + nav controls ──────────────────────────
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

                    let play_label = if self.anim_playing { "⏸" } else { "▶" };
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

                let step = &self.parse_trace[cursor];

                // ── Action label ─────────────────────────────────────────
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

                // ── State stack + remaining input ─────────────────────────
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

            },
        );
    }

    fn show_ast_formation_section(&self, ui: &mut egui::Ui, panel_h: f32) {
        if self.parse_trace.is_empty() {
            return;
        }

        let step = &self.parse_trace[self.trace_cursor];
        let total = self.parse_trace.len();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("AST Formation:").size(16.0));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "Step {} / {}",
                    self.trace_cursor + 1,
                    total
                ))
                .size(13.0)
                .color(egui::Color32::GRAY),
            );
        });
        ui.add_space(5.0);

        egui::ScrollArea::horizontal()
            .id_salt("ast_formation_hscroll")
            .show(ui, |ui| {
                ui.set_min_height(panel_h);
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

    fn show_parse_table_section(&self, ui: &mut egui::Ui) {
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

        let highlight = self.parse_trace.get(self.trace_cursor).map(|step| {
            (step.from_state, step.lookahead)
        });

        if let Some((symbols, table)) = &self.parse_table {
            self.show_parse_table(ui, symbols, table, highlight);
        } else {
            ui.label("No parse table available. Please parse a grammar first.");
        }
    }

    fn show_parse_table(
        &self,
        ui: &mut egui::Ui,
        symbols: &[char],
        table: &[Vec<ParseTableAction>],
        highlight: Option<(usize, char)>,
    ) {
        let fixed_height = 300.0;

        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), fixed_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(fixed_height)
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
        match self.selected_kind {
            ParserKind::Lr0 => self.handle_parse_lr0(),
            other => {
                self.parser_result =
                    UiError::NotImplemented(other.label().to_string()).to_string();
                self.parse_trace.clear();
                self.parse_table = None;
            }
        }
    }

    fn handle_parse_lr0(&mut self) {
        // ── フェーズ1: 独立な2チェーンを Applicative 的に合成 ──────────────────
        // grammar → compile（依存的逐次）と input parse（独立）は互いに影響しない。
        // map2 で合成し、両方成功した場合のみ RunRequest を生成する。
        // 一方または両方が失敗した場合はすべてのエラーを蓄積して表示する。
        let compiled = validated_compile(&self.reducer_string);
        let input    = validate_input(&self.input_string);

        let request = match compiled.map2(input, |cg, symbols| RunRequest {
            grammar:       cg.grammar,
            machine:       cg.machine,
            input_symbols: symbols,
        }) {
            Validation::Valid(req) => req,
            Validation::Invalid(errors) => {
                self.parser_result = errors.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.parse_table = None;
                self.parse_trace.clear();
                return;
            }
        };

        // ── フェーズ2: run（RunRequest への依存的な逐次処理） ──────────────────
        self.parse_table = Some(build_parse_table(&request.grammar, &request.machine));
        self.terminals   = terminals_from_grammar(&request.grammar);
        self.apply_default_terminal_types();

        match run(&request.machine, &request.input_symbols).map_err(UiError::Runtime) {
            Ok(_) => {
                self.parser_result.clear();
            }
            Err(e) => {
                self.parser_result = e.to_string();
                self.parse_trace.clear();
                return;
            }
        }

        self.parse_trace = build_animation_trace(&request.machine, &request.input_symbols).unwrap_or_default();
        self.trace_cursor      = 0;
        self.anim_playing      = false;
        self.anim_last_advance = None;
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
