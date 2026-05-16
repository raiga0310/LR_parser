pub mod ast;
pub mod grammar;
pub mod lr;
pub mod runtime;

pub use ast::AstNode;
pub use runtime::{ParseStep, StepAction, build_trace};

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
    fn lr0_grammar_parses_nested_angle_brackets() {
        // S -> SP | P,  P -> <> | <S>
        // 8 states, all conflict-free — confirmed LR(0)
        let grammar = parse_grammar_text("S -> SP\nS -> P\nP -> <>\nP -> <S>").unwrap();
        let machine = compile(&grammar).unwrap();
        let input = parse_input_text("<<>><>").unwrap();

        let result = run(&machine, &input).unwrap();

        assert_eq!(
            result.ast.to_string(),
            "S\n    S\n        P\n            <\n            S\n                P\n                    <\n                    >\n            >\n    P\n        <\n        >\n"
        );
    }

    #[test]
    fn paren_grammar_is_rejected_as_non_lr0() {
        // E -> EE creates a Shift/Reduce conflict: after reducing EE->E,
        // the parser cannot decide between reducing again or shifting '<'.
        let grammar = parse_grammar_text(include_str!("../paren_reducer")).unwrap();
        assert!(matches!(compile(&grammar), Err(crate::lr::ParserError::ConflictReducer)));
    }
}
