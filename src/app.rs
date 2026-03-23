use crate::generator_engine::GeneratorEngine;
use eframe::{App, egui};
use lr0_parser_rs::{Action, Parser, from_reducer_string};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// GUIアプリケーションの状態を管理する構造体
pub struct ParserApp {
    pub input_string: String,
    pub reducer_string: String,
    pub parser_result: String,
    pub terminals: Vec<char>,
    pub parser_state: Arc<Mutex<Option<Parser>>>,
    pub current_page: usize, // 現在表示中のページ (0: Parser, 1: Generator)
    pub generate_result: String,
    pub terminal_types: HashMap<char, String>, // 各終端記号のプルダウン選択状態
    pub run_result: String,                    // Rustコード実行結果
    pub generator_engine: GeneratorEngine,     // コード生成エンジン
    pub parse_table: Option<(Vec<char>, Vec<Vec<Action>>)>, // 構文解析表
}

impl Default for ParserApp {
    fn default() -> Self {
        Self {
            input_string: String::new(),
            reducer_string: String::from("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1"),
            parser_result: String::new(),
            terminals: vec![],
            parser_state: Arc::new(Mutex::new(None)),
            current_page: 0,
            generate_result: String::new(),
            terminal_types: HashMap::new(),
            run_result: String::new(),
            generator_engine: GeneratorEngine::new(),
            parse_table: None,
        }
    }
}

impl App for ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // フォントサイズを設定（スタイルを使用）
        self.setup_fonts(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左側余白を削減
                ui.add_space(5.0);

                ui.vertical(|ui| {
                    ui.add_space(10.0); // 上部の余白を削減
                    ui.heading("LR(0) Parser GUI");
                    ui.add_space(10.0); // タイトル下の余白を削減

                    // タブ選択
                    self.show_tabs(ui);

                    ui.add_space(10.0); // タブ下の余白を削減

                    // 現在のページに応じて表示を切り替え
                    match self.current_page {
                        0 => self.show_parser_page(ui),
                        1 => self.show_generator_page(ui),
                        _ => {}
                    }

                    ui.add_space(10.0); // 下部の余白を削減
                });
            });
        });
    }
}

impl ParserApp {
    // フォント設定
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

    // タブ表示
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
                self.terminals = from_reducer_string(&self.reducer_string.clone()).unwrap().1;
            }
        });
    }

    // コード生成処理
    pub fn generate_code(&mut self) {
        // GeneratorEngineに終端記号タイプを設定
        self.generator_engine
            .set_terminal_types(self.terminal_types.clone());

        // コード生成を実行
        self.generate_result = self
            .generator_engine
            .generate_code(&self.reducer_string, &self.input_string);
    }

    // 生成されたRustコードを実行
    pub fn run_rust_code(&mut self) {
        self.run_result = self.generator_engine.run_rust_code(&self.generate_result);
    }
}
