use crate::app::ParserApp;
use eframe::egui;

impl ParserApp {
    pub fn show_generator_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width() * 0.36, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Terminal Roles").size(18.0).strong());
                    ui.label(
                        egui::RichText::new(
                            "Map terminals to arithmetic roles to unlock expression generation.",
                        )
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .id_salt("terminal_symbols_area")
                        .max_height(300.0)
                        .show(ui, |ui| {
                            if self.terminals.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(
                                            "No terminals available yet.\nParse a grammar on the Parser page first.",
                                        )
                                        .size(14.0)
                                        .color(egui::Color32::GRAY),
                                    )
                                    .wrap(),
                                );
                            } else {
                                let terminals = self.terminals.clone();
                                for (index, symbol) in terminals.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{:>2}.", index + 1))
                                                .monospace()
                                                .size(14.0),
                                        );
                                        ui.label(
                                            egui::RichText::new(symbol.to_string())
                                                .monospace()
                                                .size(16.0)
                                                .strong(),
                                        );
                                        ui.add_space(10.0);
                                        self.show_terminal_dropdown(ui, *symbol);
                                    });
                                    ui.add_space(6.0);
                                }
                            }
                        });

                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("Target String").size(18.0).strong());
                    ui.add_space(8.0);
                    ui.text_edit_singleline(&mut self.input_string);

                    ui.add_space(16.0);
                    if ui
                        .button(egui::RichText::new("Generate Code").size(17.0))
                        .clicked()
                    {
                        self.generate_code();
                    }

                    ui.add_space(8.0);
                    if ui
                        .button(egui::RichText::new("Run Generated Rust").size(17.0))
                        .clicked()
                    {
                        self.run_rust_code();
                    }

                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("Notes").size(18.0).strong());
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .id_salt("generator_notes_area")
                        .max_height(160.0)
                        .show(ui, |ui| {
                            if self.generator_notes.is_empty() {
                                ui.label(
                                    egui::RichText::new(
                                        "No notes yet. Generate code to see validation hints.",
                                    )
                                    .size(13.0)
                                    .color(egui::Color32::GRAY),
                                );
                            } else {
                                for note in &self.generator_notes {
                                    ui.label(
                                        egui::RichText::new(format!("- {}", note))
                                            .size(13.0)
                                            .color(egui::Color32::LIGHT_BLUE),
                                    );
                                }
                            }
                        });
                },
            );

            ui.add_space(18.0);

            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Generation Summary").size(18.0).strong());
                    ui.add_space(8.0);
                    self.show_preview_card(ui, "AST Preview", &self.generator_ast_preview, 120.0);
                    ui.add_space(10.0);
                    self.show_preview_card(
                        ui,
                        "Reconstructed Source",
                        &self.generator_source_preview,
                        70.0,
                    );
                    ui.add_space(10.0);
                    self.show_preview_card(
                        ui,
                        "Evaluation Expression",
                        &self.generator_expression_preview,
                        70.0,
                    );

                    ui.add_space(14.0);
                    ui.label(egui::RichText::new("Generated Rust Code").size(18.0).strong());
                    ui.add_space(8.0);
                    self.show_preview_card(ui, "Code", &self.generate_result, 210.0);

                    ui.add_space(14.0);
                    ui.label(egui::RichText::new("Execution Result").size(18.0).strong());
                    ui.add_space(8.0);
                    self.show_preview_card(ui, "Run", &self.run_result, 160.0);
                },
            );
        });
    }

    fn show_preview_card(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        content: &str,
        max_height: f32,
    ) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(label)
                    .size(14.0)
                    .strong()
                    .color(egui::Color32::from_rgb(190, 190, 210)),
            );
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt(format!("{}_scroll", label))
                .max_height(max_height)
                .show(ui, |ui| {
                    if content.trim().is_empty() {
                        ui.label(
                            egui::RichText::new("Nothing yet.")
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                        );
                    } else {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(content)
                                    .monospace()
                                    .size(12.5),
                            )
                            .wrap(),
                        );
                    }
                });
        });
    }

    fn show_terminal_dropdown(&mut self, ui: &mut egui::Ui, symbol: char) {
        let current_selection = self
            .terminal_types
            .get(&symbol)
            .unwrap_or(&"Token".to_string())
            .clone();

        egui::ComboBox::from_id_salt(format!("combo_{}", symbol))
            .selected_text(&current_selection)
            .show_ui(ui, |ui| {
                let options = [
                    "Token", "Num", "Add", "Sub", "Mul", "Div", "Mod", "LParen", "RParen",
                    "Ignore",
                ];
                for option in &options {
                    ui.selectable_value(
                        self.terminal_types
                            .entry(symbol)
                            .or_insert_with(|| "Token".to_string()),
                        option.to_string(),
                        *option,
                    );
                }
            });
    }
}
