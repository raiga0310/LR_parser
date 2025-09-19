pub mod parser;

use crate::parser::Parser;
fn main() {
    let mut parser = Parser::new("./reducer");
    dbg!(parser.parse(String::from("1+1$")));
    let mut parser = Parser::new("./paren_reducer");
    dbg!(parser.parse(String::from("<><<>><>$")));
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
        assert_eq!(parser.parse(String::from("1+1$")), vec![5, 3, 5, 2]);
        assert_eq!(parser.parse(String::from("1*1$")), vec![5, 3, 5, 1]);
        assert_eq!(parser.parse(String::from("1*0+1$")), vec![5, 3, 4, 1, 5, 2]);
        assert_eq!(parser.parse(String::from("1+1*0$")), vec![5, 3, 5, 2, 4, 1]);
    }

    #[test]
    fn test_paren_parse() {
        let mut parser = Parser::new("./paren_reducer");
        assert_eq!(parser.parse(String::from("<>$")), vec![1]);
        assert_eq!(parser.parse(String::from("<<>><>$")), vec![1, 2, 1, 3]);
        assert_eq!(
            parser.parse(String::from("<><><><><><>$")),
            vec![1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3]
        );
    }
}
