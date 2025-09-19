use crate::app::ParserApp;
use eframe::egui;

impl ParserApp {
    // Generatorページの表示（4:6の比率）
    pub fn show_generator_page(&mut self, ui: &mut egui::Ui) {
        // 左右のレイアウト（4:6の比率）
        ui.horizontal(|ui| {
            // 左側パネル（40%）- 終端記号リスト
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width() * 0.4, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.add_space(15.0); // セクション間の余白
                    ui.label(egui::RichText::new("Terminal Symbols:").size(18.0));
                    ui.add_space(10.0);

                    // 終端記号リスト表示エリア
                    egui::ScrollArea::vertical().id_salt("terminal_symbols_area")
                        .min_scrolled_height(400.0)
                        .show(ui, |ui| {
                            if self.terminals.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("No terminal symbols found.\nPlease check the Parser page first.")
                                            .size(14.0)
                                            .color(egui::Color32::GRAY),
                                    )
                                    .wrap(),
                                );
                            } else {
                                // 終端記号をクローンしてループ処理
                                let terminals_clone = self.terminals.clone();
                                for (i, &symbol) in terminals_clone.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        // 番号とシンボル表示
                                        ui.label(
                                            egui::RichText::new(format!("{}.", i + 1))
                                                .size(16.0)
                                                .monospace(),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("{}", symbol))
                                                .size(16.0)
                                                .monospace()
                                                .strong(),
                                        );

                                        ui.add_space(10.0); // シンボルとプルダウンの間隔

                                        // プルダウンメニュー
                                        self.show_terminal_dropdown(ui, symbol);
                                    });
                                    ui.add_space(8.0); // 各行の間隔
                                }
                            }
                        });

                    ui.add_space(20.0); // セクション間の余白

                    // Generateボタン
                    if ui.button(egui::RichText::new("Generate Code").size(18.0)).clicked() {
                        self.generate_code();
                    }

                    ui.add_space(10.0); // ボタン間の余白

                    // Run Rust Codeボタン
                    if ui.button(egui::RichText::new("Run Rust Code").size(18.0)).clicked() {
                        self.run_rust_code();
                    }
                },
            );

            ui.add_space(20.0); // 左右のパネル間の余白

            // 右側パネル（60%）- 実行結果表示
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.add_space(15.0); // セクション間の余白
                    ui.label(egui::RichText::new("Generate Result:").size(18.0));
                    ui.add_space(10.0);

                    // 実行結果表示エリア
                    egui::ScrollArea::vertical().id_salt("generate_result_area")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            if self.generate_result.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("No generation results yet.\nClick 'Generate Code' to generate Rust code from AST.")
                                            .size(14.0)
                                            .color(egui::Color32::GRAY),
                                    )
                                    .wrap(),
                                );
                            } else {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&self.generate_result)
                                            .monospace()
                                            .size(12.0),
                                    )
                                    .wrap(),
                                );
                            }
                        });

                    ui.add_space(20.0); // セクション間の余白

                    // 実行結果セクション
                    ui.label(egui::RichText::new("Execution Result:").size(18.0));
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical().id_salt("run_result_area")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            if self.run_result.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("No execution results yet.\nClick 'Run Rust Code' to execute generated code.")
                                            .size(14.0)
                                            .color(egui::Color32::GRAY),
                                    )
                                    .wrap(),
                                );
                            } else {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&self.run_result)
                                            .monospace()
                                            .size(12.0),
                                    )
                                    .wrap(),
                                );
                            }
                        });
                },
            );
        });
    }

    // 終端記号のプルダウンメニューを表示
    fn show_terminal_dropdown(&mut self, ui: &mut egui::Ui, symbol: char) {
        let current_selection = self
            .terminal_types
            .get(&symbol)
            .unwrap_or(&"Token".to_string())
            .clone();

        egui::ComboBox::from_id_salt(format!("combo_{}", symbol))
            .selected_text(&current_selection)
            .show_ui(ui, |ui| {
                let options = ["Add", "Mul", "L_paren", "R_paren", "Num"];
                for option in &options {
                    if ui
                        .selectable_value(
                            self.terminal_types
                                .entry(symbol)
                                .or_insert("Token".to_string()),
                            option.to_string(),
                            *option,
                        )
                        .clicked()
                    {
                        // 選択が変更された時の処理（必要に応じて）
                    }
                }
            });
    }
}
