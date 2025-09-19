use lr0_parser_rs::{AstNode, Parser};
use std::collections::HashMap;

/// コード生成エンジン - AST からRustコードの生成と実行を担当
pub struct GeneratorEngine {
    pub terminal_types: HashMap<char, String>,
}

impl GeneratorEngine {
    pub fn new() -> Self {
        Self {
            terminal_types: HashMap::new(),
        }
    }

    /// 終端記号のタイプを設定
    pub fn set_terminal_types(&mut self, terminal_types: HashMap<char, String>) {
        self.terminal_types = terminal_types;
    }

    /// ASTからRustコードを生成
    pub fn generate_code(&self, reducer_string: &str, input_string: &str) -> String {
        match Parser::new_from_string(reducer_string) {
            Ok(mut parser) => {
                let ast_nodes = parser.parse(input_string.to_owned() + "$");

                if ast_nodes.is_empty() {
                    return String::from(
                        "No AST available. Please parse first or check your input.",
                    );
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
                generated_code
            }
            Err(e) => {
                format!("Failed to create parser for code generation: {}", e)
            }
        }
    }

    /// ASTから数式表現を生成
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
                            // 二項演算（E -> E + B または E -> E * B）
                            let left = self.generate_expression_from_ast(&children[0]);
                            let operator = self.get_operator_from_ast(&children[1]);
                            let right = self.generate_expression_from_ast(&children[2]);
                            format!("({} {} {})", left, operator, right)
                        } else {
                            // その他の構成
                            format!("unknown_structure_{}", symbol)
                        }
                    }
                    _ => {
                        // その他の非終端記号
                        if children.len() == 1 {
                            self.generate_expression_from_ast(&children[0])
                        } else {
                            format!("unknown_nonterminal_{}", symbol)
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

    /// ASTから演算子を取得
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

    /// 生成されたRustコードを実行
    pub fn run_rust_code(&self, generated_code: &str) -> String {
        if generated_code.is_empty() {
            return "No code to run. Please generate code first.".to_string();
        }

        "Not implemented".to_string()
    }
}

impl Default for GeneratorEngine {
    fn default() -> Self {
        Self::new()
    }
}
