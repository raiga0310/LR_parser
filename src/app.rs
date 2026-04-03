use crate::generator_engine::GeneratorEngine;
use eframe::{App, egui};
use lr0_parser_rs::grammar::{Grammar, parse_grammar_text};
use lr0_parser_rs::lr::{self, CompiledParser};
use std::collections::HashMap;

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

pub struct ParserApp {
    pub input_string: String,
    pub reducer_string: String,
    pub parser_result: String,
    pub terminals: Vec<char>,
    pub current_page: usize,
    pub generate_result: String,
    pub generator_ast_preview: String,
    pub generator_source_preview: String,
    pub generator_expression_preview: String,
    pub generator_notes: Vec<String>,
    pub terminal_types: HashMap<char, String>,
    pub run_result: String,
    pub generator_engine: GeneratorEngine,
    pub parse_table: Option<(Vec<char>, Vec<Vec<ParseTableAction>>)>,
}

impl Default for ParserApp {
    fn default() -> Self {
        Self {
            input_string: String::new(),
            reducer_string: String::from("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1"),
            parser_result: String::new(),
            terminals: vec![],
            current_page: 0,
            generate_result: String::new(),
            generator_ast_preview: String::new(),
            generator_source_preview: String::new(),
            generator_expression_preview: String::new(),
            generator_notes: Vec::new(),
            terminal_types: HashMap::new(),
            run_result: String::new(),
            generator_engine: GeneratorEngine::new(),
            parse_table: None,
        }
    }
}

impl App for ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_fonts(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(5.0);

                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("LR(0) Parser GUI");
                    ui.add_space(10.0);

                    self.show_tabs(ui);

                    ui.add_space(10.0);

                    match self.current_page {
                        0 => self.show_parser_page(ui),
                        1 => self.show_generator_page(ui),
                        _ => {}
                    }

                    ui.add_space(10.0);
                });
            });
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
                .selectable_label(self.current_page == 0, "Parser")
                .clicked()
            {
                self.current_page = 0;
            }
            if ui
                .selectable_label(self.current_page == 1, "Generator")
                .clicked()
            {
                self.current_page = 1;
                self.terminals = parse_grammar_text(&self.reducer_string)
                    .map(|grammar| terminals_from_grammar(&grammar))
                    .unwrap_or_default();
                self.apply_default_terminal_types();
            }
        });
    }

    pub fn generate_code(&mut self) {
        self.generator_engine
            .set_terminal_types(self.terminal_types.clone());
        match self
            .generator_engine
            .generate_output(&self.reducer_string, &self.input_string)
        {
            Ok(output) => {
                self.generator_ast_preview = output.ast_preview;
                self.generator_source_preview = output.source_preview;
                self.generator_expression_preview =
                    output.evaluation_expression.unwrap_or_else(|| "<not available>".to_string());
                self.generator_notes = output.notes;
                self.generate_result = output.generated_code;
                self.run_result.clear();
            }
            Err(err) => {
                self.generator_ast_preview.clear();
                self.generator_source_preview.clear();
                self.generator_expression_preview.clear();
                self.generator_notes = vec![err.clone()];
                self.generate_result = err;
                self.run_result.clear();
            }
        }
    }

    pub fn run_rust_code(&mut self) {
        self.run_result = self.generator_engine.run_rust_code(&self.generate_result);
    }

    pub fn apply_default_terminal_types(&mut self) {
        for &terminal in &self.terminals {
            self.terminal_types
                .entry(terminal)
                .or_insert_with(|| default_terminal_role(terminal).to_string());
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
    let mut symbols: Vec<char> = grammar.terminals().into_iter().map(|terminal| terminal.0).collect();
    if !symbols.contains(&'$') {
        symbols.push('$');
    }
    symbols.extend(
        grammar
            .non_terminals()
            .into_iter()
            .map(|non_terminal| non_terminal.0),
    );

    let mut table = vec![vec![ParseTableAction::Error; symbols.len()]; compiled_parser.state_count()];

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
