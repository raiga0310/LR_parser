use std::fmt;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AstNode {
    Terminal(char),
    NonTerminal(char, Vec<AstNode>),
}

// ASTNode の表示用補助関数
fn print_ast(node: &AstNode, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match node {
        AstNode::Terminal(c) => {
            writeln!(f, "{:indent$}{}", "", c, indent = indent * 4)?;
        }
        AstNode::NonTerminal(name, children) => {
            writeln!(f, "{:indent$}{}", "", name, indent = indent * 4)?;
            for child in children {
                print_ast(child, f, indent + 1)?;
            }
        }
    }
    Ok(())
}

// Displayトレイトの実装
impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_ast(self, f, 0)
    }
}
