use eframe::{App, NativeOptions, egui};
use lr0_parser_rs::Parser;
use std::sync::{Arc, Mutex};

// GUIアプリケーションの状態を管理する構造体
struct ParserApp {
    input_string: String,
    reducer_string: String,
    parser_result: String,
    parser_state: Arc<Mutex<Option<Parser>>>,
}

impl Default for ParserApp {
    fn default() -> Self {
        Self {
            input_string: String::new(),
            reducer_string: String::from("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1"),
            parser_result: String::new(),
            parser_state: Arc::new(Mutex::new(None)),
        }
    }
}

impl App for ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // フォントサイズを設定（スタイルを使用）
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左側余白を追加
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.add_space(20.0); // 上部の余白
                    ui.heading("LR(0) Parser GUI");
                    ui.add_space(20.0); // タイトル下の余白

                    // 左右のレイアウト（3:7の比率）
                    ui.horizontal(|ui| {
                        // 左側パネル（30%）
                        ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width() * 0.3, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(15.0); // セクション間の余白
                        ui.label(egui::RichText::new("Production:").size(18.0));
                        ui.add_space(10.0);

                        // Production入力エリア
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(ui.available_width(), 200.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                egui::ScrollArea::vertical()
                                    .min_scrolled_height(200.0)
                                    .show(ui, |ui| {
                                        ui.text_edit_multiline(&mut self.reducer_string);
                                    });
                            },
                        );

                        ui.add_space(20.0); // セクション間の余白

                        ui.label(egui::RichText::new("Target String:").size(18.0));
                        ui.add_space(10.0);
                        ui.text_edit_singleline(&mut self.input_string);

                        ui.add_space(20.0); // ボタン上の余白

                        if ui.button(egui::RichText::new("Parse").size(18.0)).clicked() {
                            // Parserのインスタンスを生成して状態に保存
                            let parser_state_clone = self.parser_state.clone();
                            let reducer_clone = self.reducer_string.clone();
                            let input_clone = self.input_string.clone();

                            let mut parser_state = parser_state_clone.lock().unwrap();

                            match Parser::new_from_string(&reducer_clone) {
                                Ok(mut parser) => {
                                    let ast_nodes = parser.parse(input_clone);
                                    self.parser_result = if ast_nodes.is_empty() {
                                        String::from("Failed to parse or invalid inputs")
                                    } else {
                                        ast_nodes
                                            .iter()
                                            .map(|node| node.to_string())
                                            .collect::<String>()
                                    };
                                    *parser_state = Some(parser);
                                }
                                Err(_) => {
                                    self.parser_result = String::from(
                                        "Failed to generate parser. Check your productions.",
                                    );
                                }
                            }
                        }
                    },
                );

                        ui.add_space(20.0); // 左右のパネル間の余白

                        // 右側パネル（70%）
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(ui.available_width(), ui.available_height()),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.add_space(15.0); // セクション間の余白
                                ui.label(egui::RichText::new("Parse Result (AST):").size(18.0));
                                ui.add_space(10.0);

                                // 結果表示エリア
                                egui::ScrollArea::vertical()
                                    .min_scrolled_height(400.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&self.parser_result)
                                                    .monospace()
                                                    .size(14.0),
                                            )
                                            .wrap(),
                                        );
                                    });
                            },
                        );
                    });

                    ui.add_space(20.0); // 下部の余白
                });
            });
        });
    }
}

// GUIアプリケーションのエントリーポイント
fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "LR(0) Parser",
        options,
        Box::new(|_cc| Ok(Box::<ParserApp>::default())),
    )
}
