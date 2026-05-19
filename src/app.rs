use crate::generator_engine::GeneratorEngine;
use eframe::{App, egui};
use lr0_parser_rs::grammar::{Grammar, parse_grammar_text};
use lr0_parser_rs::lr::{self, CompiledParser};
use lr0_parser_rs::{AstNode, ParseStep, StateInfo, StepAction, build_trace};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ── Parse result types ────────────────────────────────────────────────────────

pub struct ParseArtifacts {
    pub symbols: Vec<char>,
    pub table: Vec<Vec<ParseTableAction>>,
    pub state_infos: Vec<StateInfo>,
    pub accept_states: Vec<usize>,
}

pub enum ParserStatus {
    Empty,
    Ready(ParseArtifacts),
}

// ── View models (read-only DTOs for UI rendering) ────────────────────────────

#[derive(Clone)]
pub struct TraceCursorView {
    pub cursor: usize,
    pub total: usize,
    pub is_playing: bool,
    pub step: Option<ParseStep>,
}

#[derive(Clone, Default)]
pub struct SmHighlightView {
    pub active_edge: Option<(usize, char, usize)>,
    pub source_state: Option<usize>,
    pub result_state: Option<usize>,
}

impl TraceCursorView {
    pub fn sm_highlight(&self) -> SmHighlightView {
        let Some(step) = &self.step else {
            return SmHighlightView::default();
        };
        let active_edge = match &step.action {
            StepAction::Shift { terminal, to_state } => {
                Some((step.from_state, *terminal, *to_state))
            }
            StepAction::Reduce { .. } | StepAction::Accept => None,
        };
        let source_state = Some(step.from_state);
        let result_state = match &step.action {
            StepAction::Shift { to_state, .. } => Some(*to_state),
            StepAction::Reduce { .. } => step.state_stack.last().copied(),
            StepAction::Accept => None,
        };
        SmHighlightView { active_edge, source_state, result_state }
    }
}

// ── Page / algorithm enums ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Parser,
    Generator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserKind {
    Lr0,
    Slr,
    Lalr,
    Lr1,
}

impl ParserKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lr0 => "LR(0)",
            Self::Slr => "SLR(1)",
            Self::Lalr => "LALR(1)",
            Self::Lr1 => "LR(1)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseTableAction {
    Shift(usize),
    Reduce(usize),
    Accept,
    Goto(usize),
    Error,
}

impl ParseTableAction {
    pub fn as_label(self) -> String {
        match self {
            Self::Shift(state) => format!("s{}", state),
            Self::Reduce(production) => format!("r{}", production),
            Self::Accept => "acc".to_string(),
            Self::Goto(state) => format!("g{}", state),
            Self::Error => String::new(),
        }
    }
}

// ── Sub-state structs ─────────────────────────────────────────────────────────

pub struct UiState {
    pub current_page: Page,
    pub fonts_initialized: bool,
}

pub struct AppServices {
    pub generator_engine: GeneratorEngine,
}

pub struct WorkspaceState {
    pub input_string: String,
    pub reducer_string: String,
    pub terminals: Vec<char>,
    pub terminal_types: HashMap<char, String>,
}

pub struct GeneratorPageState {
    pub generate_result: String,
    pub ast_preview: String,
    pub source_preview: String,
    pub expression_preview: String,
    pub notes: Vec<String>,
    pub run_result: String,
    pub ast: Option<AstNode>,
}

pub struct ParserPageState {
    pub result: String,
    pub selected_kind: ParserKind,
    pub parse_trace: Vec<ParseStep>,
    pub trace_cursor: usize,
    pub anim_playing: bool,
    pub anim_last_advance: Option<Instant>,
    pub status: ParserStatus,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct ParserApp {
    pub ui: UiState,
    pub services: AppServices,
    pub workspace: WorkspaceState,
    pub generator: GeneratorPageState,
    pub parser: ParserPageState,
}

impl Default for ParserApp {
    fn default() -> Self {
        Self {
            ui: UiState {
                current_page: Page::Parser,
                fonts_initialized: false,
            },
            services: AppServices {
                generator_engine: GeneratorEngine::new(),
            },
            workspace: WorkspaceState {
                input_string: String::new(),
                reducer_string: String::from("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1"),
                terminals: vec![],
                terminal_types: HashMap::new(),
            },
            generator: GeneratorPageState {
                generate_result: String::new(),
                ast_preview: String::new(),
                source_preview: String::new(),
                expression_preview: String::new(),
                notes: Vec::new(),
                run_result: String::new(),
                ast: None,
            },
            parser: ParserPageState {
                result: String::new(),
                selected_kind: ParserKind::Lr0,
                parse_trace: Vec::new(),
                trace_cursor: 0,
                anim_playing: false,
                anim_last_advance: None,
                status: ParserStatus::Empty,
            },
        }
    }
}

impl App for ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.parser.anim_playing {
            if let Some(last) = self.parser.anim_last_advance {
                if last.elapsed() >= Duration::from_millis(700) {
                    if self.parser.trace_cursor + 1 < self.parser.parse_trace.len() {
                        self.parser.trace_cursor += 1;
                        self.parser.anim_last_advance = Some(Instant::now());
                    } else {
                        self.parser.anim_playing = false;
                    }
                }
            }
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        if !self.ui.fonts_initialized {
            self.setup_fonts(ctx);
            self.ui.fonts_initialized = true;
        }

        egui::TopBottomPanel::top("app_header")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("LR(0) Parser GUI");
                ui.add_space(10.0);
                self.show_tabs(ui);
                ui.add_space(10.0);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.ui.current_page {
                Page::Parser => self.show_parser_page(ui),
                Page::Generator => self.show_generator_page(ui),
            }
        });
    }
}

impl ParserApp {
    fn setup_fonts(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
        ]
        .into();
        ctx.set_style(style);
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.ui.current_page == Page::Parser, "Parser")
                .clicked()
            {
                self.ui.current_page = Page::Parser;
            }
            if ui
                .selectable_label(self.ui.current_page == Page::Generator, "Generator")
                .clicked()
            {
                self.ui.current_page = Page::Generator;
                self.workspace.terminals = parse_grammar_text(&self.workspace.reducer_string)
                    .map(|grammar| terminals_from_grammar(&grammar))
                    .unwrap_or_default();
                self.apply_default_terminal_types();
            }
        });
    }

    pub fn generate_code(&mut self) {
        self.services
            .generator_engine
            .set_terminal_types(self.workspace.terminal_types.clone());
        match self
            .services
            .generator_engine
            .generate_output(&self.workspace.reducer_string, &self.workspace.input_string)
        {
            Ok(output) => {
                self.generator.ast_preview = output.ast_preview;
                self.generator.ast = output.ast;
                self.generator.source_preview = output.source_preview;
                self.generator.expression_preview =
                    output.evaluation_expression.unwrap_or_else(|| "<not available>".to_string());
                self.generator.notes = output.notes;
                self.generator.generate_result = output.generated_code;
                self.generator.run_result.clear();
            }
            Err(err) => {
                self.generator.ast_preview.clear();
                self.generator.ast = None;
                self.generator.source_preview.clear();
                self.generator.expression_preview.clear();
                self.generator.notes = vec![err.clone()];
                self.generator.generate_result = err;
                self.generator.run_result.clear();
            }
        }
    }

    pub fn run_rust_code(&mut self) {
        let code = self.generator.generate_result.clone();
        self.generator.run_result = crate::generator_engine::run_generated_code(&code);
    }

    pub fn apply_default_terminal_types(&mut self) {
        for &terminal in &self.workspace.terminals {
            self.workspace
                .terminal_types
                .entry(terminal)
                .or_insert_with(|| default_terminal_role(terminal).to_string());
        }
    }

    pub fn cursor_view(&self) -> TraceCursorView {
        TraceCursorView {
            cursor: self.parser.trace_cursor,
            total: self.parser.parse_trace.len(),
            is_playing: self.parser.anim_playing,
            step: self.parser.parse_trace.get(self.parser.trace_cursor).cloned(),
        }
    }

    pub fn step_to(&mut self, cursor: usize) {
        self.parser.trace_cursor =
            cursor.min(self.parser.parse_trace.len().saturating_sub(1));
        self.parser.anim_playing = false;
    }

    pub fn toggle_play(&mut self) {
        if self.parser.parse_trace.is_empty() {
            return;
        }
        self.parser.anim_playing = !self.parser.anim_playing;
        if self.parser.anim_playing {
            if self.parser.trace_cursor + 1 >= self.parser.parse_trace.len() {
                self.parser.trace_cursor = 0;
            }
            self.parser.anim_last_advance = Some(Instant::now());
        }
    }
}

pub fn default_terminal_role(symbol: char) -> &'static str {
    match symbol {
        '+' => "Add",
        '-' => "Sub",
        '*' => "Mul",
        '/' => "Div",
        '%' => "Mod",
        '(' | '<' | '[' | '{' => "LParen",
        ')' | '>' | ']' | '}' => "RParen",
        '0'..='9' => "Num",
        _ => "Token",
    }
}

pub fn terminals_from_grammar(grammar: &Grammar) -> Vec<char> {
    grammar
        .terminals()
        .into_iter()
        .filter_map(|terminal| (terminal.0 != '$').then_some(terminal.0))
        .collect()
}

pub fn build_parse_table(
    grammar: &Grammar,
    compiled_parser: &CompiledParser,
) -> (Vec<char>, Vec<Vec<ParseTableAction>>) {
    let mut symbols: Vec<char> =
        grammar.terminals().into_iter().map(|terminal| terminal.0).collect();
    if !symbols.contains(&'$') {
        symbols.push('$');
    }
    symbols.extend(
        grammar
            .non_terminals()
            .into_iter()
            .map(|non_terminal| non_terminal.0),
    );

    let mut table =
        vec![vec![ParseTableAction::Error; symbols.len()]; compiled_parser.state_count()];

    for state in 0..compiled_parser.state_count() {
        for (column, symbol) in symbols.iter().copied().enumerate() {
            table[state][column] = if symbol == '$' || !symbol.is_ascii_uppercase() {
                compiled_parser
                    .action(state, lr0_parser_rs::grammar::Terminal(symbol))
                    .map(|action| match action {
                        lr::Action::Shift(next) => ParseTableAction::Shift(next),
                        lr::Action::Reduce(production) => ParseTableAction::Reduce(production),
                        lr::Action::Accept => ParseTableAction::Accept,
                    })
                    .unwrap_or(ParseTableAction::Error)
            } else {
                compiled_parser
                    .goto(state, lr0_parser_rs::grammar::NonTerminal(symbol))
                    .map(ParseTableAction::Goto)
                    .unwrap_or(ParseTableAction::Error)
            };
        }
    }

    (symbols, table)
}

pub fn build_animation_trace(
    machine: &CompiledParser,
    input: &[lr0_parser_rs::grammar::Symbol],
) -> Option<Vec<ParseStep>> {
    build_trace(machine, input).ok()
}
