use eframe::{App, egui};
use lr0_parser_rs::{Parser, from_reducer_string};
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
        }
    }
}

impl App for ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // フォントサイズを設定（スタイルを使用）
        self.setup_fonts(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左側余白を追加
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.add_space(20.0); // 上部の余白
                    ui.heading("LR(0) Parser GUI");
                    ui.add_space(20.0); // タイトル下の余白

                    // タブ選択
                    self.show_tabs(ui);

                    ui.add_space(20.0);

                    // 現在のページに応じて表示を切り替え
                    match self.current_page {
                        0 => self.show_parser_page(ui),
                        1 => self.show_generator_page(ui),
                        _ => {}
                    }

                    ui.add_space(20.0); // 下部の余白
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
}
