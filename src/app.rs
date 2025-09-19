use eframe::{App, egui};
use lr0_parser_rs::{AstNode, Parser, from_reducer_string};
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

    // コード生成処理
    pub fn generate_code(&mut self) {
        // 新しいパーサーを作成して解析
        match Parser::new_from_string(&self.reducer_string) {
            Ok(mut parser) => {
                let ast_nodes = parser.parse(self.input_string.clone() + "$");

                if ast_nodes.is_empty() {
                    self.generate_result =
                        String::from("No AST available. Please parse first or check your input.");
                    return;
                }

                // AST全体からコードを生成
                let mut generated_code = String::new();
                generated_code.push_str("// Generated Rust code from AST\n");
                generated_code.push_str("fn main() {\n");

                for (i, ast_node) in ast_nodes.iter().enumerate() {
                    let var_name = format!("result_{}", i + 1);
                    let expression = self.generate_expression_from_ast(ast_node);
                    generated_code.push_str(&format!(
                        "    let {} = {}; // Evaluated expression\n",
                        var_name, expression
                    ));
                    generated_code
                        .push_str(&format!("    println!(\"Result: {{}}\", {});\n", var_name));
                }

                generated_code.push_str("}\n");
                self.generate_result = generated_code;
            }
            Err(e) => {
                self.generate_result =
                    format!("Failed to create parser for code generation: {}", e);
            }
        }
    }

    // ASTから数式表現を生成
    fn generate_expression_from_ast(&self, node: &AstNode) -> String {
        match node {
            AstNode::NonTerminal(symbol, children) => {
                match *symbol {
                    'E' | 'B' => {
                        // 式の場合、子ノードの構成を確認
                        if children.len() == 1 {
                            // 単一の子ノード（E -> B または B -> 数値）
                            self.generate_expression_from_ast(&children[0])
                        } else if children.len() == 3 {
                            // 3つの子ノード（E -> E op B）
                            let left = self.generate_expression_from_ast(&children[0]);
                            let operator = self.get_operator_from_ast(&children[1]);
                            let right = self.generate_expression_from_ast(&children[2]);

                            if operator.is_empty() {
                                format!("({} {} {})", left, "?", right)
                            } else {
                                format!("({} {} {})", left, operator, right)
                            }
                        } else {
                            // その他の構成
                            format!("unknown_expr({})", children.len())
                        }
                    }
                    _ => {
                        // その他の非終端記号
                        if !children.is_empty() {
                            self.generate_expression_from_ast(&children[0])
                        } else {
                            format!("empty_{}", symbol)
                        }
                    }
                }
            }
            AstNode::Terminal(symbol) => {
                // 終端記号の場合
                let default_type = "Token".to_string();
                let token_type = self.terminal_types.get(symbol).unwrap_or(&default_type);

                match token_type.as_str() {
                    "Num" => {
                        // 数値の場合、そのまま返す
                        symbol.to_string()
                    }
                    _ => {
                        // その他の場合は変数として扱う
                        format!("token_{}", symbol)
                    }
                }
            }
        }
    }

    // ASTから演算子を取得
    fn get_operator_from_ast(&self, node: &AstNode) -> String {
        match node {
            AstNode::Terminal(symbol) => {
                let default_type = "Token".to_string();
                let token_type = self.terminal_types.get(symbol).unwrap_or(&default_type);

                match token_type.as_str() {
                    "Add" => "+".to_string(),
                    "Mul" => "*".to_string(),
                    "L_paren" => "(".to_string(),
                    "R_paren" => ")".to_string(),
                    "Token" => "".to_string(),
                    _ => symbol.to_string(),
                }
            }
            _ => String::new(),
        }
    }
}
