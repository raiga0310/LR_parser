use lr0_parser_rs::AstNode;
use lr0_parser_rs::grammar::{parse_grammar_text, parse_input_text};
use lr0_parser_rs::lr::compile;
use lr0_parser_rs::runtime::run;
use std::collections::HashMap;

pub struct GeneratorEngine {
    pub terminal_types: HashMap<char, String>,
}

impl GeneratorEngine {
    pub fn new() -> Self {
        Self {
            terminal_types: HashMap::new(),
        }
    }

    pub fn set_terminal_types(&mut self, terminal_types: HashMap<char, String>) {
        self.terminal_types = terminal_types;
    }

    pub fn generate_code(&self, reducer_string: &str, input_string: &str) -> String {
        let grammar = match parse_grammar_text(reducer_string) {
            Ok(grammar) => grammar,
            Err(err) => return format!("Failed to parse grammar: {err:?}"),
        };

        let machine = match compile(&grammar) {
            Ok(machine) => machine,
            Err(err) => return format!("Failed to compile parser: {err:?}"),
        };

        let input = match parse_input_text(input_string) {
            Ok(input) => input,
            Err(err) => return format!("Failed to parse input: {err:?}"),
        };

        let result = match run(&machine, &input) {
            Ok(result) => result,
            Err(err) => return format!("Failed to run parser: {err:?}"),
        };

        let mut generated_code = String::new();
        generated_code.push_str("// Generated Rust code from AST\n");
        generated_code.push_str("fn main() {\n");

        let expression = self.generate_expression_from_ast(&result.ast);
        generated_code.push_str(&format!(
            "    let result_1 = {}; // Evaluated expression\n",
            expression
        ));
        generated_code.push_str("    println!(\"Result: {}\", result_1);\n");
        generated_code.push_str("}\n");
        generated_code
    }

    fn generate_expression_from_ast(&self, node: &AstNode) -> String {
        match node {
            AstNode::NonTerminal(symbol, children) => match *symbol {
                'E' | 'B' => {
                    if children.len() == 1 {
                        self.generate_expression_from_ast(&children[0])
                    } else if children.len() == 3 {
                        let left = self.generate_expression_from_ast(&children[0]);
                        let operator = self.get_operator_from_ast(&children[1]);
                        let right = self.generate_expression_from_ast(&children[2]);
                        format!("({} {} {})", left, operator, right)
                    } else {
                        format!("unknown_structure_{}", symbol)
                    }
                }
                _ => {
                    if children.len() == 1 {
                        self.generate_expression_from_ast(&children[0])
                    } else {
                        format!("unknown_nonterminal_{}", symbol)
                    }
                }
            },
            AstNode::Terminal(symbol) => {
                let default_type = "Token".to_string();
                let token_type = self.terminal_types.get(symbol).unwrap_or(&default_type);

                match token_type.as_str() {
                    "Num" => symbol.to_string(),
                    _ => format!("token_{}", symbol),
                }
            }
        }
    }

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
                    "Token" => String::new(),
                    _ => symbol.to_string(),
                }
            }
            _ => String::new(),
        }
    }

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
