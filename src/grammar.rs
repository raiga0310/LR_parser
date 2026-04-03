use std::collections::BTreeSet;

pub(crate) fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

pub(crate) fn parse_grammar_text(input: &str) -> Result<Grammar, GrammarError> {
    let mut productions = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split("->");
        let left = parts
            .next()
            .ok_or(GrammarError::InvalidProductionFormat)?
            .trim();
        let right = parts
            .next()
            .ok_or(GrammarError::InvalidProductionFormat)?
            .trim();

        if parts.next().is_some() {
            return Err(GrammarError::InvalidProductionFormat);
        }

        let left = left
            .chars()
            .next()
            .ok_or(GrammarError::MissingLeftHandSide)?;

        if !left.is_ascii_uppercase() {
            return Err(GrammarError::InvalidSymbol(left));
        }

        let right = right
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(Symbol::from_char)
            .collect::<Result<Vec<_>, _>>()?;

        productions.push(Production {
            left: NonTerminal(left),
            right,
        });
    }

    let start = productions
        .first()
        .map(|production| production.left)
        .ok_or(GrammarError::EmptyGrammar)?;

    Ok(Grammar { start, productions })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Grammar {
    pub start: NonTerminal,
    pub productions: Vec<Production>,
}

impl Grammar {
    pub(crate) fn non_terminals(&self) -> BTreeSet<NonTerminal> {
        let mut set = BTreeSet::new();
        set.insert(self.start);
        for production in &self.productions {
            set.insert(production.left);
            for symbol in &production.right {
                if let Symbol::NonTerminal(nt) = symbol {
                    set.insert(*nt);
                }
            }
        }
        set
    }

    pub(crate) fn terminals(&self) -> BTreeSet<Terminal> {
        let mut set = BTreeSet::new();
        for production in &self.productions {
            for symbol in &production.right {
                if let Symbol::Terminal(tt) = symbol {
                    set.insert(*tt);
                }
            }
        }
        set
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarError {
    EmptyGrammar,
    InvalidProductionFormat,
    MissingLeftHandSide,
    InvalidSymbol(char),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Production {
    pub left: NonTerminal,
    pub right: Vec<Symbol>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

impl Symbol {
    fn from_char(value: char) -> Result<Self, GrammarError> {
        if value.is_whitespace() {
            return Err(GrammarError::InvalidSymbol(value));
        }

        if value.is_ascii_uppercase() {
            Ok(Self::NonTerminal(NonTerminal(value)))
        } else {
            Ok(Self::Terminal(Terminal(value)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terminal(pub char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonTerminal(pub char);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_grammar_text_builds_start_and_productions() {
        let grammar = parse_grammar_text("E -> E+B\nE -> B\nB -> 0").unwrap();

        assert_eq!(grammar.start, NonTerminal('E'));
        assert_eq!(grammar.productions.len(), 3);
    }
}
