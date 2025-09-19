use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
    Goto(usize),
    Error,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Production {
    left: char,
    right: Vec<char>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Item {
    production: Production,
    dot_pos: usize,
}

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

pub struct Parser {
    stack: Vec<usize>,
    state: usize,
    table: (Vec<char>, Vec<Vec<Action>>),
    reducer: Vec<(char, String)>,
    ast_stack: Vec<AstNode>,
}

impl Parser {
    pub fn new_from_string(reducer_str: &str) -> Result<Self, String> {
        let reducer_map = from_reducer_string(reducer_str)?;
        let (symbols, table) = Self::generate_lr0_table(&reducer_map);
        Ok(Parser {
            stack: vec![],
            state: 0,
            table: (symbols, table),
            reducer: reducer_map,
            ast_stack: vec![],
        })
    }

    pub fn parse(&mut self, mut input: String) -> Vec<AstNode> {
        self.stack.clear();
        self.stack.push(0);
        self.state = 0;
        self.ast_stack.clear();
        let mut chars: Vec<char> = input.chars().collect();
        input.clear();

        while let Some(head) = chars.first().cloned() {
            let action = self.action(head);
            match action {
                Action::Shift(id) => {
                    self.stack.push(id);
                    self.ast_stack.push(AstNode::Terminal(head));
                    chars.remove(0);
                    self.state = id;
                }
                Action::Reduce(id) => {
                    let (num_pop, result) = self.apply_reducer(id);
                    let children: Vec<AstNode> = self
                        .ast_stack
                        .drain(self.ast_stack.len() - num_pop..)
                        .collect();
                    let reversed_children = children.into_iter().rev().collect::<Vec<_>>();

                    self.ast_stack
                        .push(AstNode::NonTerminal(result, reversed_children));

                    for _ in 0..num_pop {
                        let _ = self.stack.pop();
                    }
                    self.state = *self.stack.last().unwrap();
                    if let Action::Goto(next) = self.action(result) {
                        self.stack.push(next);
                    }
                    self.state = *self.stack.last().unwrap();
                }
                Action::Accept => {
                    break;
                }
                _ => return vec![],
            };
        }
        self.ast_stack.drain(..).collect()
    }

    // ... その他の補助メソッド（generate_lr0_table, parse_grammar, closure, goto, action, apply_reducer）
    fn generate_lr0_table(reducer_map: &Vec<(char, String)>) -> (Vec<char>, Vec<Vec<Action>>) {
        let (mut productions, non_terminals, terminals) = Self::parse_grammar(reducer_map);
        let start_symbol = productions[0].left;
        let augmented_start = (start_symbol as u8 + 1) as char;
        productions.insert(
            0,
            Production {
                left: augmented_start,
                right: vec![start_symbol],
            },
        );

        let mut items_sets: Vec<HashSet<Item>> = vec![];
        let mut edges: HashMap<(usize, char), usize> = HashMap::new();

        let initial_item = Item {
            production: productions[0].clone(),
            dot_pos: 0,
        };
        let initial_closure = Self::closure(
            &vec![initial_item].into_iter().collect(),
            &productions,
            &non_terminals,
        );
        items_sets.push(initial_closure);

        let mut i = 0;
        while i < items_sets.len() {
            let current_set = items_sets[i].clone();
            let mut all_symbols = terminals.clone();
            all_symbols.extend(non_terminals.clone());

            for &symbol in &all_symbols {
                let goto_set = Self::goto(&current_set, symbol, &productions, &non_terminals);
                if !goto_set.is_empty() {
                    let mut found_idx = None;
                    for (idx, existing_set) in items_sets.iter().enumerate() {
                        if *existing_set == goto_set {
                            found_idx = Some(idx);
                            break;
                        }
                    }

                    let next_state_id = if let Some(idx) = found_idx {
                        idx
                    } else {
                        let new_id = items_sets.len();
                        items_sets.push(goto_set);
                        new_id
                    };
                    edges.insert((i, symbol), next_state_id);
                }
            }
            i += 1;
        }

        let mut symbols = terminals.clone();
        symbols.insert('$');
        symbols.extend(non_terminals.clone());

        let mut table = vec![vec![Action::Error; symbols.len()]; items_sets.len()];

        for (i, item_set) in items_sets.iter().enumerate() {
            for item in item_set {
                if item.dot_pos < item.production.right.len() {
                    let next_symbol = item.production.right[item.dot_pos];
                    if let Some(&next_state) = edges.get(&(i, next_symbol)) {
                        let col_idx = symbols.iter().position(|&s| s == next_symbol).unwrap();
                        if terminals.contains(&next_symbol) {
                            if table[i][col_idx] == Action::Error {
                                table[i][col_idx] = Action::Shift(next_state);
                            }
                        } else if table[i][col_idx] == Action::Error {
                            table[i][col_idx] = Action::Goto(next_state);
                        }
                    }
                } else {
                    let prod_idx = productions
                        .iter()
                        .position(|p| {
                            p.left == item.production.left && p.right == item.production.right
                        })
                        .unwrap();
                    if prod_idx == 0 {
                        let col_idx = symbols.iter().position(|&s| s == '$').unwrap();
                        table[i][col_idx] = Action::Accept;
                    } else {
                        for (col_idx, &symbol) in symbols.iter().enumerate() {
                            if (terminals.contains(&symbol) || symbol == '$')
                                && table[i][col_idx] == Action::Error
                            {
                                table[i][col_idx] = Action::Reduce(prod_idx);
                            }
                        }
                    }
                }
            }
        }
        let symbols: Vec<char> = symbols.into_iter().collect();
        (symbols, table)
    }

    fn parse_grammar(
        reducer_map: &Vec<(char, String)>,
    ) -> (Vec<Production>, HashSet<char>, HashSet<char>) {
        let mut productions = vec![];
        let mut non_terminals = HashSet::new();
        let mut terminals = HashSet::new();
        let mut all_symbols = HashSet::new();

        for (left, right) in reducer_map {
            non_terminals.insert(*left);
            all_symbols.insert(*left);
            let prod = Production {
                left: *left,
                right: right.chars().collect(),
            };
            productions.push(prod);
        }

        for prod in &productions {
            for &symbol in &prod.right {
                all_symbols.insert(symbol);
            }
        }

        for symbol in &all_symbols {
            if !non_terminals.contains(symbol) {
                terminals.insert(*symbol);
            }
        }

        (productions, non_terminals, terminals)
    }

    fn closure(
        items: &HashSet<Item>,
        productions: &[Production],
        non_terminals: &HashSet<char>,
    ) -> HashSet<Item> {
        let mut closure_set = items.clone();
        let mut new_items_to_add = closure_set.clone();

        loop {
            let mut added_this_iteration = false;
            let current_items_to_process = new_items_to_add.clone();
            new_items_to_add.clear();

            for item in &current_items_to_process {
                if item.dot_pos < item.production.right.len() {
                    let next_symbol = item.production.right[item.dot_pos];
                    if non_terminals.contains(&next_symbol) {
                        for prod in productions.iter().filter(|p| p.left == next_symbol) {
                            let new_item = Item {
                                production: prod.clone(),
                                dot_pos: 0,
                            };
                            if closure_set.insert(new_item.clone()) {
                                new_items_to_add.insert(new_item);
                                added_this_iteration = true;
                            }
                        }
                    }
                }
            }
            if !added_this_iteration {
                break;
            }
        }
        closure_set
    }

    fn goto(
        items: &HashSet<Item>,
        symbol: char,
        productions: &[Production],
        non_terminals: &HashSet<char>,
    ) -> HashSet<Item> {
        let mut next_items = HashSet::new();
        for item in items {
            if item.dot_pos < item.production.right.len()
                && item.production.right[item.dot_pos] == symbol
            {
                let mut next_item = item.clone();
                next_item.dot_pos += 1;
                next_items.insert(next_item);
            }
        }
        Self::closure(&next_items, productions, non_terminals)
    }

    fn action(&self, input: char) -> Action {
        let idx = self.state;
        let (symbols, table) = self.table.clone();
        let actions = table[idx].clone();
        let idx = symbols.iter().position(|&s| s == input).unwrap();
        actions[idx]
    }

    fn apply_reducer(&self, id: usize) -> (usize, char) {
        let (after, before) = &self.reducer[id - 1];
        (before.len(), *after)
    }
}

pub fn from_reducer_string(content: &str) -> Result<Vec<(char, String)>, String> {
    let mut reducer = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let condition: Vec<&str> = line.trim().split("->").collect();
        if condition.len() != 2 {
            return Err(format!("Invalid reducer format: {}", line));
        }
        let (before, after) = (
            condition[0]
                .chars()
                .next()
                .ok_or("Invalid left-hand side")?,
            condition[1]
                .trim()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
        );
        reducer.push((before, after));
    }
    Ok(reducer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_reducer() {
        let reducer = from_reducer_string(include_str!("../test_reducer")).unwrap();
        assert_eq!(reducer[0], ('A', String::from("A+A")));
    }

    #[test]
    fn test_parse() {
        let mut parser = Parser::new_from_string(include_str!("../reducer")).unwrap();
        let result = parser.parse(String::from("1+1$"));
        assert_eq!(
            result,
            vec![AstNode::NonTerminal(
                'E',
                vec![
                    AstNode::NonTerminal('B', vec![AstNode::Terminal('1'),],),
                    AstNode::Terminal('+'),
                    AstNode::NonTerminal(
                        'E',
                        vec![AstNode::NonTerminal('B', vec![AstNode::Terminal('1'),],),],
                    ),
                ],
            ),]
        );
        let result = parser.parse(String::from("1+1*1$"));
        assert_eq!(
            result,
            vec![AstNode::NonTerminal(
                'E',
                vec![
                    AstNode::NonTerminal('B', vec![AstNode::Terminal('1')],),
                    AstNode::Terminal('*'),
                    AstNode::NonTerminal(
                        'E',
                        vec![
                            AstNode::NonTerminal('B', vec![AstNode::Terminal('1')],),
                            AstNode::Terminal('+'),
                            AstNode::NonTerminal(
                                'E',
                                vec![AstNode::NonTerminal('B', vec![AstNode::Terminal('1')],)],
                            ),
                        ],
                    ),
                ],
            )]
        );
    }

    #[test]
    fn test_paren_parse() {
        let mut parser = Parser::new_from_string(include_str!("../paren_reducer")).unwrap();
        let result = parser.parse(String::from("<<>><>$"));
        assert_eq!(
            result,
            vec![AstNode::NonTerminal(
                'E',
                vec![
                    AstNode::NonTerminal(
                        'E',
                        vec![AstNode::Terminal('>'), AstNode::Terminal('<'),],
                    ),
                    AstNode::NonTerminal(
                        'E',
                        vec![
                            AstNode::Terminal('>'),
                            AstNode::NonTerminal(
                                'E',
                                vec![AstNode::Terminal('>'), AstNode::Terminal('<'),],
                            ),
                            AstNode::Terminal('<'),
                        ],
                    ),
                ],
            )]
        );
    }
}
