pub mod parser;

use crate::parser::Parser;
fn main() {
    let mut parser = Parser::new("./reducer");
    println!("{}", parser.parse(String::from("1*1+1$"))[0]);
    let mut parser = Parser::new("./paren_reducer");
    println!("{}", parser.parse(String::from("<><<>><>$"))[0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_reducer() {
        let reducer = parser::from_reducer("./test_reducer");
        assert_eq!(reducer[0], ('A', String::from("A+A")));
    }

    // === B: ブラックボックステスト ===

    #[test]
    fn parse_result_is_deterministic() {
        let inputs = ["1+1$", "1*1+1$", "0+1*0$", "1$", "0$"];
        for input in inputs {
            let r1 = Parser::new("./reducer").parse(input.to_string());
            let r2 = Parser::new("./reducer").parse(input.to_string());
            assert_eq!(r1, r2, "非決定: input={input}");
        }
    }

    #[test]
    fn parse_reuse_same_instance_is_stable() {
        let mut parser = Parser::new("./reducer");
        let r1 = parser.parse("1+1$".to_string());
        let r2 = parser.parse("1+1$".to_string());
        assert_eq!(r1, r2);
    }

    #[test]
    fn invalid_input_returns_empty() {
        let mut parser = Parser::new("./reducer");
        assert!(parser.parse("1++1$".to_string()).is_empty());
        assert!(parser.parse("+$".to_string()).is_empty());
    }

    #[test]
    fn paren_parse_result_is_deterministic() {
        let inputs = ["<>$", "<<>>$", "<><>$", "<><><>$"];
        for input in inputs {
            let r1 = Parser::new("./paren_reducer").parse(input.to_string());
            let r2 = Parser::new("./paren_reducer").parse(input.to_string());
            assert_eq!(r1, r2, "非決定: input={input}");
        }
    }

    // === A: ホワイトボックステスト（原因特定用） ===

    #[test]
    fn table_symbols_order_is_deterministic() {
        let p1 = Parser::new("./paren_reducer");
        let p2 = Parser::new("./paren_reducer");
        assert_eq!(
            p1.get_symbols(),
            p2.get_symbols(),
            "symbolsの順序が毎回異なる"
        );
    }

    #[test]
    fn table_rows_are_deterministic() {
        let p1 = Parser::new("./paren_reducer");
        let p2 = Parser::new("./paren_reducer");
        let (_, t1) = p1.get_table();
        let (_, t2) = p2.get_table();
        assert_eq!(t1, t2, "テーブルの内容が毎回異なる");
    }

    #[test]
    fn table_has_dollar_column() {
        let p = Parser::new("./reducer");
        assert!(p.get_symbols().contains(&'$'), "$列が存在しない");
    }

    #[test]
    fn table_has_exactly_one_accept() {
        let p = Parser::new("./reducer");
        let (_, table) = p.get_table();
        let accepts = table
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&a| a == parser::Action::Accept)
            .count();
        assert_eq!(accepts, 1, "Acceptが{}個ある", accepts);
    }

    // === 既存テスト ===

    #[test]
    fn test_parse() {
        let mut parser = Parser::new("./reducer");
        let result = parser.parse(String::from("1+1$"));
        assert_eq!(
            result,
            vec![parser::AstNode::NonTerminal(
                'E',
                vec![
                    parser::AstNode::NonTerminal('B', vec![parser::AstNode::Terminal('1'),],),
                    parser::AstNode::Terminal('+'),
                    parser::AstNode::NonTerminal(
                        'E',
                        vec![parser::AstNode::NonTerminal(
                            'B',
                            vec![parser::AstNode::Terminal('1'),],
                        ),],
                    ),
                ],
            ),]
        );
        let result = parser.parse(String::from("1+1*1$"));
        assert_eq!(
            result,
            vec![parser::AstNode::NonTerminal(
                'E',
                vec![
                    parser::AstNode::NonTerminal('B', vec![parser::AstNode::Terminal('1')],),
                    parser::AstNode::Terminal('*'),
                    parser::AstNode::NonTerminal(
                        'E',
                        vec![
                            parser::AstNode::NonTerminal('B', vec![parser::AstNode::Terminal('1')],),
                            parser::AstNode::Terminal('+'),
                            parser::AstNode::NonTerminal(
                                'E',
                                vec![parser::AstNode::NonTerminal(
                                    'B',
                                    vec![parser::AstNode::Terminal('1')],
                                )],
                            ),
                        ],
                    ),
                ],
            )]
        );
    }

    #[test]
    fn test_paren_parse() {
        let mut parser = Parser::new("./paren_reducer");
        let result = parser.parse(String::from("<<>><>$"));
        assert_eq!(
            result,
            vec![parser::AstNode::NonTerminal(
                'E',
                vec![
                    parser::AstNode::NonTerminal(
                        'E',
                        vec![
                            parser::AstNode::Terminal('>'),
                            parser::AstNode::Terminal('<'),
                        ],
                    ),
                    parser::AstNode::NonTerminal(
                        'E',
                        vec![
                            parser::AstNode::Terminal('>'),
                            parser::AstNode::NonTerminal(
                                'E',
                                vec![
                                    parser::AstNode::Terminal('>'),
                                    parser::AstNode::Terminal('<'),
                                ],
                            ),
                            parser::AstNode::Terminal('<'),
                        ],
                    ),
                ],
            )]
        );
    }
}
