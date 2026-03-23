use eframe::egui;
use lr0_parser_rs::{Action, Parser, from_reducer_string};

use crate::app::ParserApp;

impl ParserApp {
    // Parserページの表示
    pub fn show_parser_page(&mut self, ui: &mut egui::Ui) {
        // 上下のレイアウト
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // 左側パネル（35%）
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width() * 0.35, 280.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Production:").size(16.0));
                        ui.add_space(5.0);

                        // Production入力エリア（高さを削減）
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(ui.available_width(), 150.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                egui::ScrollArea::vertical()
                                    .min_scrolled_height(150.0)
                                    .show(ui, |ui| {
                                        ui.text_edit_multiline(&mut self.reducer_string);
                                    });
                            },
                        );

                        ui.add_space(15.0);
                        ui.label(egui::RichText::new("Target String:").size(16.0));
                        ui.add_space(5.0);
                        ui.text_edit_singleline(&mut self.input_string);

                        ui.add_space(15.0);
                        if ui.button(egui::RichText::new("Parse").size(16.0)).clicked() {
                            self.handle_parse();
                        }
                    },
                );

                ui.add_space(15.0);

                // 右側パネル（65%）AST結果表示
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 280.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Parse Result (AST):").size(16.0));
                        ui.add_space(5.0);

                        // 結果表示エリア（高さを削減）
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(240.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&self.parser_result)
                                            .monospace()
                                            .size(13.0),
                                    )
                                    .wrap(),
                                );
                            });
                    },
                );
            });

            ui.add_space(15.0); // 上下のセクション間の余白を削減

            // 下部：構文解析表表示
            ui.label(egui::RichText::new("Parse Table:").size(16.0));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Legend:").size(11.0).strong());
                ui.label(
                    egui::RichText::new("s<n>")
                        .color(egui::Color32::from_rgb(0, 120, 0))
                        .monospace()
                        .size(11.0),
                );
                ui.label("=Shift,");
                ui.label(
                    egui::RichText::new("r<n>")
                        .color(egui::Color32::from_rgb(0, 0, 120))
                        .monospace()
                        .size(11.0),
                );
                ui.label("=Reduce,");
                ui.label(
                    egui::RichText::new("g<n>")
                        .color(egui::Color32::from_rgb(120, 60, 0))
                        .monospace()
                        .size(11.0),
                );
                ui.label("=Goto,");
                ui.label(
                    egui::RichText::new("acc")
                        .color(egui::Color32::from_rgb(120, 0, 120))
                        .monospace()
                        .size(11.0),
                );
                ui.label("=Accept");
            });
            ui.add_space(5.0);

            if let Some((symbols, table)) = &self.parse_table {
                self.show_parse_table(ui, symbols, table);
            } else {
                ui.label("No parse table available. Please parse a grammar first.");
            }
        });
    }

    // 構文解析表を表示する関数
    fn show_parse_table(&self, ui: &mut egui::Ui, symbols: &[char], table: &[Vec<Action>]) {
        // 縦幅を200に固定
        let fixed_height = 300.0;

        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), fixed_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                // スクロールエリアを使用して固定縦幅内に収める
                egui::ScrollArea::vertical()
                    .max_height(fixed_height)
                    .show(ui, |ui| {
                        self.render_table_grid(ui, symbols, table);
                    });
            },
        );
    }

    // テーブルグリッドを描画する補助関数
    fn render_table_grid(&self, ui: &mut egui::Ui, symbols: &[char], table: &[Vec<Action>]) {
        egui::Grid::new("parse_table")
            .num_columns(symbols.len() + 1)
            .spacing([25.0, 3.0]) // スペーシングをより密に
            .striped(true)
            .show(ui, |ui| {
                // ヘッダー行
                ui.label(egui::RichText::new("State").strong().size(13.0));
                for symbol in symbols {
                    ui.label(egui::RichText::new(symbol.to_string()).strong().size(13.0));
                }
                ui.end_row();

                // 各状態の行
                for (state_id, actions) in table.iter().enumerate() {
                    ui.label(
                        egui::RichText::new(state_id.to_string())
                            .monospace()
                            .size(11.0),
                    );
                    for action in actions {
                        let action_str = Parser::action_to_string(action);
                        let color = match action {
                            Action::Shift(_) => egui::Color32::from_rgb(0, 120, 0), // 緑
                            Action::Reduce(_) => egui::Color32::from_rgb(0, 0, 120), // 青
                            Action::Accept => egui::Color32::from_rgb(120, 0, 120), // 紫
                            Action::Goto(_) => egui::Color32::from_rgb(120, 60, 0), // 橙
                            Action::Error => egui::Color32::from_rgb(100, 100, 100), // 灰
                        };
                        if action_str.is_empty() {
                            ui.label(
                                egui::RichText::new("-")
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(action_str)
                                    .monospace()
                                    .size(11.0)
                                    .color(color),
                            );
                        }
                    }
                    ui.end_row();
                }
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
                // 構文解析表を保存
                self.parse_table = Some(parser.get_parse_table());

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
                self.parse_table = None;
            }
        }
    }
}
