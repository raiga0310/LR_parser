use crate::app::ParserApp;
use eframe::egui;
use super::tree::{draw_tree, layout_ast, tree_pixel_height, NODE_R};

impl ParserApp {
    pub fn show_generator_page(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("gen_page_scroll")
            .show(ui, |ui| {
                let left_w = ui.available_width() * 0.36;
                ui.horizontal_top(|ui| {
                    // ── 左列 ──────────────────────────────────────────────
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
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

                        if self.workspace.terminals.is_empty() {
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
                            let terminals = self.workspace.terminals.clone();
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

                        ui.add_space(16.0);
                        ui.label(egui::RichText::new("Target String").size(18.0).strong());
                        ui.add_space(8.0);
                        ui.text_edit_singleline(&mut self.workspace.input_string);

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
                        if self.generator.notes.is_empty() {
                            ui.label(
                                egui::RichText::new(
                                    "No notes yet. Generate code to see validation hints.",
                                )
                                .size(13.0)
                                .color(egui::Color32::GRAY),
                            );
                        } else {
                            for note in &self.generator.notes {
                                ui.label(
                                    egui::RichText::new(format!("- {}", note))
                                        .size(13.0)
                                        .color(egui::Color32::LIGHT_BLUE),
                                );
                            }
                        }
                    });

                    ui.add_space(18.0);

                    // ── 右列 ──────────────────────────────────────────────
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Generation Summary").size(18.0).strong());
                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("AST Preview")
                                .size(14.0)
                                .strong()
                                .color(egui::Color32::from_rgb(190, 190, 210)),
                        );
                        ui.add_space(6.0);
                        if let Some(ast) = self.generator.ast.clone() {
                            let layout = layout_ast(&ast);
                            let tree_w = layout.subtree_width.max(60.0);
                            let tree_h = tree_pixel_height(&layout) + NODE_R * 2.0 + 4.0;
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                egui::ScrollArea::both()
                                    .id_salt("gen_ast_scroll")
                                    .max_height(tree_h)
                                    .show(ui, |ui| {
                                        let size = egui::Vec2::new(tree_w, tree_h);
                                        let (rect, _) =
                                            ui.allocate_exact_size(size, egui::Sense::hover());
                                        if ui.is_rect_visible(rect) {
                                            draw_tree(
                                                ui.painter(),
                                                rect.min + egui::Vec2::new(0.0, NODE_R),
                                                &layout,
                                            );
                                        }
                                    });
                            });
                        } else {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new("Nothing yet.")
                                        .size(13.0)
                                        .color(egui::Color32::GRAY),
                                );
                            });
                        }

                        ui.add_space(10.0);
                        self.show_preview_card(
                            ui,
                            "Reconstructed Source",
                            &self.generator.source_preview.clone(),
                        );
                        ui.add_space(10.0);
                        self.show_preview_card(
                            ui,
                            "Evaluation Expression",
                            &self.generator.expression_preview.clone(),
                        );

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new("Generated Rust Code").size(18.0).strong());
                        ui.add_space(8.0);
                        self.show_preview_card(ui, "Code", &self.generator.generate_result.clone());

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new("Execution Result").size(18.0).strong());
                        ui.add_space(8.0);
                        self.show_preview_card(ui, "Run", &self.generator.run_result.clone());
                    });
                });
            });
    }

    fn show_preview_card(&self, ui: &mut egui::Ui, label: &str, content: &str) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(label)
                    .size(14.0)
                    .strong()
                    .color(egui::Color32::from_rgb(190, 190, 210)),
            );
            ui.add_space(6.0);
            if content.trim().is_empty() {
                ui.label(
                    egui::RichText::new("Nothing yet.")
                        .size(13.0)
                        .color(egui::Color32::GRAY),
                );
            } else {
                egui::ScrollArea::vertical()
                    .id_salt(format!("preview_card_{}", label))
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(content).monospace().size(12.5),
                            )
                            .wrap(),
                        );
                    });
            }
        });
    }

    fn show_terminal_dropdown(&mut self, ui: &mut egui::Ui, symbol: char) {
        let current_selection = self
            .workspace.terminal_types
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
                        self.workspace.terminal_types
                            .entry(symbol)
                            .or_insert_with(|| "Token".to_string()),
                        option.to_string(),
                        *option,
                    );
                }
            });
    }
}
