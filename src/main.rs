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
