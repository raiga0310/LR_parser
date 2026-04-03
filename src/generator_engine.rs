use lr0_parser_rs::AstNode;
use lr0_parser_rs::grammar::{parse_grammar_text, parse_input_text};
use lr0_parser_rs::lr::compile;
use lr0_parser_rs::runtime::run;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationOutput {
    pub ast_preview: String,
    pub source_preview: String,
    pub evaluation_expression: Option<String>,
    pub generated_code: String,
    pub notes: Vec<String>,
}

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

    pub fn generate_output(
        &self,
        reducer_string: &str,
        input_string: &str,
    ) -> Result<GenerationOutput, String> {
        let grammar = parse_grammar_text(reducer_string)
            .map_err(|err| format!("Failed to parse grammar: {err:?}"))?;
        let machine =
            compile(&grammar).map_err(|err| format!("Failed to compile parser: {err:?}"))?;
        let input = parse_input_text(input_string)
            .map_err(|err| format!("Failed to parse input: {err:?}"))?;
        let result =
            run(&machine, &input).map_err(|err| format!("Failed to run parser: {err:?}"))?;

        let ast_preview = result.ast.to_string();
        let source_preview = self.render_source_preview(&result.ast);
        let evaluation_expression = self.render_evaluation_expression(&result.ast);

        let mut notes = Vec::new();
        if source_preview.is_empty() {
            notes.push("The reconstructed source preview is empty.".to_string());
        }
        if evaluation_expression.is_none() {
            notes.push(
                "Assign arithmetic roles such as Num/Add/Mul/LParen/RParen to generate an evaluable Rust expression."
                    .to_string(),
            );
        }
        if self.terminal_types.is_empty() {
            notes.push(
                "No terminal role overrides are set yet, so raw terminal characters are used."
                    .to_string(),
            );
        }

        let generated_code = self.build_rust_program(
            &ast_preview,
            &source_preview,
            evaluation_expression.as_deref(),
        );

        Ok(GenerationOutput {
            ast_preview,
            source_preview,
            evaluation_expression,
            generated_code,
            notes,
        })
    }

    fn render_source_preview(&self, node: &AstNode) -> String {
        let mut output = String::new();
        self.push_rendered_terminals(node, &mut output, false);
        output
    }

    fn render_evaluation_expression(&self, node: &AstNode) -> Option<String> {
        let mut output = String::new();
        self.push_rendered_terminals(node, &mut output, true)
            .then_some(output)
    }

    fn push_rendered_terminals(
        &self,
        node: &AstNode,
        output: &mut String,
        arithmetic_only: bool,
    ) -> bool {
        match node {
            AstNode::Terminal(symbol) => {
                let rendered = if arithmetic_only {
                    self.render_terminal_for_evaluation(*symbol)
                } else {
                    Some(self.render_terminal_for_source(*symbol))
                };

                let Some(rendered) = rendered else {
                    return false;
                };
                output.push_str(&rendered);
                true
            }
            AstNode::NonTerminal(_, children) => children
                .iter()
                .all(|child| self.push_rendered_terminals(child, output, arithmetic_only)),
        }
    }

    fn render_terminal_for_source(&self, symbol: char) -> String {
        match self.terminal_role(symbol).as_str() {
            "Ignore" => String::new(),
            "LParen" | "L_paren" => "(".to_string(),
            "RParen" | "R_paren" => ")".to_string(),
            _ => symbol.to_string(),
        }
    }

    fn render_terminal_for_evaluation(&self, symbol: char) -> Option<String> {
        let rendered = match self.terminal_role(symbol).as_str() {
            "Num" => symbol.to_string(),
            "Add" => "+".to_string(),
            "Sub" => "-".to_string(),
            "Mul" => "*".to_string(),
            "Div" => "/".to_string(),
            "Mod" => "%".to_string(),
            "LParen" | "L_paren" => "(".to_string(),
            "RParen" | "R_paren" => ")".to_string(),
            "Ignore" => String::new(),
            _ => return None,
        };
        Some(rendered)
    }

    fn terminal_role(&self, symbol: char) -> String {
        self.terminal_types
            .get(&symbol)
            .cloned()
            .unwrap_or_else(|| "Token".to_string())
    }

    fn build_rust_program(
        &self,
        ast_preview: &str,
        source_preview: &str,
        evaluation_expression: Option<&str>,
    ) -> String {
        let escaped_ast = escape_rust_string(ast_preview);
        let escaped_source = escape_rust_string(source_preview);

        let mut generated = String::new();
        generated.push_str("// Generated by the LR(0) Parser GUI\n");
        generated.push_str("fn generated_ast() -> &'static str {\n");
        generated.push_str(&format!("    \"{}\"\n", escaped_ast));
        generated.push_str("}\n\n");
        generated.push_str("fn generated_source() -> &'static str {\n");
        generated.push_str(&format!("    \"{}\"\n", escaped_source));
        generated.push_str("}\n\n");
        generated.push_str("fn main() {\n");
        generated.push_str("    println!(\"AST:\\n{}\", generated_ast());\n");
        generated.push_str("    println!(\"Source preview: {}\", generated_source());\n");

        if let Some(expression) = evaluation_expression {
            generated.push_str(&format!("    let value = {};\n", expression));
            generated.push_str("    println!(\"Evaluated value: {}\", value);\n");
        } else {
            generated.push_str(
                "    println!(\"Evaluated value: <not available for current terminal mappings>\");\n",
            );
        }

        generated.push_str("}\n");
        generated
    }

    pub fn run_rust_code(&self, generated_code: &str) -> String {
        if generated_code.trim().is_empty() {
            return "No code to run. Please generate code first.".to_string();
        }

        let build_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("generated");

        if let Err(err) = fs::create_dir_all(&build_dir) {
            return format!("Failed to create build directory: {err}");
        }

        let source_path = build_dir.join("generated_main.rs");
        let binary_path = build_dir.join(format!("generated_program{}", std::env::consts::EXE_SUFFIX));

        if let Err(err) = fs::write(&source_path, generated_code) {
            return format!("Failed to write generated source: {err}");
        }

        let compile_output = match Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary_path)
            .output()
        {
            Ok(output) => output,
            Err(err) => return format!("Failed to invoke rustc: {err}"),
        };

        if !compile_output.status.success() {
            return format!(
                "Rust compilation failed.\n{}",
                String::from_utf8_lossy(&compile_output.stderr)
            );
        }

        let run_output = match Command::new(&binary_path).output() {
            Ok(output) => output,
            Err(err) => return format!("Failed to run generated binary: {err}"),
        };

        let stdout = String::from_utf8_lossy(&run_output.stdout);
        let stderr = String::from_utf8_lossy(&run_output.stderr);

        if run_output.status.success() {
            if stderr.trim().is_empty() {
                stdout.trim_end().to_string()
            } else {
                format!("{}\n\nstderr:\n{}", stdout.trim_end(), stderr.trim_end())
            }
        } else if stdout.trim().is_empty() {
            format!("Generated binary failed.\n{}", stderr.trim_end())
        } else {
            format!(
                "Generated binary failed.\nstdout:\n{}\n\nstderr:\n{}",
                stdout.trim_end(),
                stderr.trim_end()
            )
        }
    }
}

fn escape_rust_string(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

impl Default for GeneratorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_output_builds_source_preview() {
        let mut engine = GeneratorEngine::new();
        engine.terminal_types.insert('+', "Add".to_string());
        engine.terminal_types.insert('*', "Mul".to_string());
        engine.terminal_types.insert('1', "Num".to_string());
        engine.terminal_types.insert('0', "Num".to_string());

        let output = engine
            .generate_output("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1", "1+0")
            .unwrap();

        assert!(output.source_preview.contains("1+0"));
        assert_eq!(output.evaluation_expression.as_deref(), Some("1+0"));
    }
}
