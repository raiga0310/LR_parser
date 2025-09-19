use eframe::egui;
use lr0_parser_rs::{Parser, from_reducer_string};

use crate::app::ParserApp;

impl ParserApp {
    // Parserページの表示
    pub fn show_parser_page(&mut self, ui: &mut egui::Ui) {
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
                        self.handle_parse();
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
    }

    // Parse処理
    fn handle_parse(&mut self) {
        // Parserのインスタンスを生成して状態に保存
        let parser_state_clone = self.parser_state.clone();
        let reducer_clone = self.reducer_string.clone();
        let input_clone = self.input_string.clone();

        let mut parser_state = parser_state_clone.lock().unwrap();

        match Parser::new_from_string(&reducer_clone) {
            Ok(mut parser) => {
                // 入力文字列の末尾に$（EOF記号）を追加
                let input_with_eof = format!("{}$", input_clone);
                let ast_nodes = parser.parse(input_with_eof);
                self.parser_result = if ast_nodes.is_empty() {
                    String::from("Failed to parse or invalid inputs")
                } else {
                    ast_nodes
                        .iter()
                        .map(|node| node.to_string())
                        .collect::<String>()
                };
                *parser_state = Some(parser);

                // 終端記号を更新
                self.terminals = from_reducer_string(&reducer_clone).unwrap().1;

                // 新しい終端記号に対してデフォルトのプルダウン選択を設定
                for &terminal in &self.terminals {
                    self.terminal_types
                        .entry(terminal)
                        .or_insert_with(|| "Token".to_string());
                }
            }
            Err(_) => {
                self.parser_result =
                    String::from("Failed to generate parser. Check your productions.");
            }
        }
    }
}
