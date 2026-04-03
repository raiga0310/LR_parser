use eframe::egui;
use lr0_parser_rs::grammar::{parse_grammar_text, parse_input_text};
use lr0_parser_rs::lr::compile;
use lr0_parser_rs::runtime::run;

use crate::app::{ParseTableAction, ParserApp, build_parse_table, terminals_from_grammar};

impl ParserApp {
    pub fn show_parser_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width() * 0.35, 280.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Production:").size(16.0));
                        ui.add_space(5.0);

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

                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 280.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Parse Result (AST):").size(16.0));
                        ui.add_space(5.0);

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

            ui.add_space(15.0);

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

    fn show_parse_table(
        &self,
        ui: &mut egui::Ui,
        symbols: &[char],
        table: &[Vec<ParseTableAction>],
    ) {
        let fixed_height = 300.0;

        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), fixed_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(fixed_height)
                    .show(ui, |ui| {
                        self.render_table_grid(ui, symbols, table);
                    });
            },
        );
    }

    fn render_table_grid(
        &self,
        ui: &mut egui::Ui,
        symbols: &[char],
        table: &[Vec<ParseTableAction>],
    ) {
        egui::Grid::new("parse_table")
            .num_columns(symbols.len() + 1)
            .spacing([25.0, 3.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("State").strong().size(13.0));
                for symbol in symbols {
                    ui.label(egui::RichText::new(symbol.to_string()).strong().size(13.0));
                }
                ui.end_row();

                for (state_id, actions) in table.iter().enumerate() {
                    ui.label(
                        egui::RichText::new(state_id.to_string())
                            .monospace()
                            .size(11.0),
                    );
                    for action in actions {
                        let action_str = action.as_label();
                        let color = match action {
                            ParseTableAction::Shift(_) => egui::Color32::from_rgb(0, 120, 0),
                            ParseTableAction::Reduce(_) => egui::Color32::from_rgb(0, 0, 120),
                            ParseTableAction::Accept => egui::Color32::from_rgb(120, 0, 120),
                            ParseTableAction::Goto(_) => egui::Color32::from_rgb(120, 60, 0),
                            ParseTableAction::Error => egui::Color32::from_rgb(100, 100, 100),
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

    fn handle_parse(&mut self) {
        match parse_grammar_text(&self.reducer_string) {
            Ok(grammar) => match compile(&grammar) {
                Ok(machine) => {
                    self.parse_table = Some(build_parse_table(&grammar, &machine));
                    self.terminals = terminals_from_grammar(&grammar);

                    for &terminal in &self.terminals {
                        self.terminal_types
                            .entry(terminal)
                            .or_insert_with(|| "Token".to_string());
                    }

                    match parse_input_text(&self.input_string).and_then(|symbols| {
                        run(&machine, &symbols)
                            .map_err(|_| lr0_parser_rs::grammar::GrammarError::InvalidProductionFormat)
                    }) {
                        Ok(result) => {
                            self.parser_result = result.ast.to_string();
                        }
                        Err(_) => {
                            self.parser_result = String::from("Failed to parse or invalid inputs");
                        }
                    }
                }
                Err(_) => {
                    self.parser_result =
                        String::from("Failed to compile parser. Check your productions.");
                    self.parse_table = None;
                }
            },
            Err(_) => {
                self.parser_result = String::from("Failed to parse grammar. Check your productions.");
                self.parse_table = None;
            }
        }
    }
}
