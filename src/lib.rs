pub mod ast;
pub mod grammar;
pub mod lr;
pub mod runtime;

pub use ast::AstNode;

#[cfg(test)]
mod tests {
    use crate::ast::AstNode;
    use crate::grammar::{parse_grammar_text, parse_input_text};
    use crate::lr::compile;
    use crate::runtime::run;

    #[test]
    fn grammar_compile_runtime_pipeline_parses_expression() {
        let grammar = parse_grammar_text(include_str!("../reducer")).unwrap();
        let machine = compile(&grammar).unwrap();
        let input = parse_input_text("1+1*1").unwrap();

        let result = run(&machine, &input).unwrap();

        assert_eq!(
            result.ast,
            AstNode::NonTerminal(
                'E',
                vec![
                    AstNode::NonTerminal(
                        'E',
                        vec![
                            AstNode::NonTerminal(
                                'E',
                                vec![AstNode::NonTerminal('B', vec![AstNode::Terminal('1')])],
                            ),
                            AstNode::Terminal('+'),
                            AstNode::NonTerminal('B', vec![AstNode::Terminal('1')]),
                        ],
                    ),
                    AstNode::Terminal('*'),
                    AstNode::NonTerminal('B', vec![AstNode::Terminal('1')]),
                ],
            )
        );
    }

    #[test]
    fn grammar_compile_runtime_pipeline_parses_parens() {
        let grammar = parse_grammar_text(include_str!("../paren_reducer")).unwrap();
        let machine = compile(&grammar).unwrap();
        let input = parse_input_text("<<>><>").unwrap();

        let result = run(&machine, &input).unwrap();

        assert_eq!(
            result.ast.to_string(),
            "E\n    E\n        <\n        E\n            <\n            >\n        >\n    E\n        <\n        >\n"
        );
    }
}
