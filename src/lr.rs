use std::collections::{BTreeMap, BTreeSet};

use crate::grammar::{Grammar, NonTerminal, Production, Symbol, Terminal};

pub(crate) fn compile(grammar: &Grammar) -> Result<CompiledParser, ParserError> {
    let mut productions = grammar.productions.clone();
    let start_symbol = grammar.start;
    let augmented_start = NonTerminal(((start_symbol.0 as u8) + 1) as char);

    productions.insert(
        0,
        Production {
            left: augmented_start,
            right: vec![Symbol::NonTerminal(start_symbol)],
        },
    );

    let non_terminals = grammar.non_terminals();
    let terminals = grammar.terminals();

    let mut item_sets: Vec<BTreeSet<Item>> = Vec::new();
    let mut edges: BTreeMap<(InternalState, Symbol), InternalState> = BTreeMap::new();

    let initial_item = Item {
        production: productions[0].clone(),
        dot_pos: 0,
    };

    let initial_closure = closure(
        &vec![initial_item].into_iter().collect(),
        &productions,
        &non_terminals,
    );
    item_sets.push(initial_closure);

    let mut state_id = 0;
    while state_id < item_sets.len() {
        let current_set = item_sets[state_id].clone();
        let mut all_symbols = Vec::new();
        all_symbols.extend(terminals.iter().copied().map(Symbol::Terminal));
        all_symbols.extend(non_terminals.iter().copied().map(Symbol::NonTerminal));

        for symbol in all_symbols {
            let goto_set = goto(&current_set, symbol.clone(), &productions, &non_terminals);
            if goto_set.is_empty() {
                continue;
            }

            let next_state =
                if let Some(existing) = item_sets.iter().position(|set| *set == goto_set) {
                    existing
                } else {
                    let new_id = item_sets.len();
                    item_sets.push(goto_set);
                    new_id
                };

            edges.insert((state_id, symbol), next_state);
        }

        state_id += 1;
    }

    let mut action_table = BTreeMap::new();
    let mut goto_table = BTreeMap::new();

    for (state_id, item_set) in item_sets.iter().enumerate() {
        for item in item_set {
            if item.dot_pos < item.production.right.len() {
                let next_symbol = item.production.right[item.dot_pos].clone();
                let Some(&next_state) = edges.get(&(state_id, next_symbol.clone())) else {
                    continue;
                };

                match next_symbol {
                    Symbol::Terminal(terminal) => {
                        insert_action(
                            &mut action_table,
                            state_id,
                            terminal,
                            Action::Shift(next_state),
                        )?;
                    }
                    Symbol::NonTerminal(non_terminal) => {
                        goto_table.insert((state_id, non_terminal), next_state);
                    }
                }
            } else {
                let production_id = productions
                    .iter()
                    .position(|production| *production == item.production)
                    .ok_or(ParserError::MissingProduction)?;

                if production_id == 0 {
                    insert_action(&mut action_table, state_id, Terminal('$'), Action::Accept)?;
                } else {
                    for terminal in &terminals {
                        insert_action(
                            &mut action_table,
                            state_id,
                            *terminal,
                            Action::Reduce(production_id),
                        )?;
                    }

                    insert_action(
                        &mut action_table,
                        state_id,
                        Terminal('$'),
                        Action::Reduce(production_id),
                    )?;
                }
            }
        }
    }

    Ok(CompiledParser {
        productions,
        action_table,
        goto_table,
        start_state: 0,
    })
}

fn insert_action(
    table: &mut BTreeMap<(InternalState, Terminal), Action>,
    state: InternalState,
    terminal: Terminal,
    action: Action,
) -> Result<(), ParserError> {
    if let std::collections::btree_map::Entry::Vacant(entry) = table.entry((state, terminal)) {
        entry.insert(action);
    }

    Ok(())
}

fn closure(
    items: &BTreeSet<Item>,
    productions: &[Production],
    non_terminals: &BTreeSet<NonTerminal>,
) -> BTreeSet<Item> {
    let mut closure_set = items.clone();
    let mut new_items_to_add = closure_set.clone();

    loop {
        let mut added_this_round = false;
        let current_items = new_items_to_add.clone();
        new_items_to_add.clear();

        for item in &current_items {
            if item.dot_pos >= item.production.right.len() {
                continue;
            }

            let next_symbol = item.production.right[item.dot_pos].clone();
            let Symbol::NonTerminal(non_terminal) = next_symbol else {
                continue;
            };

            if !non_terminals.contains(&non_terminal) {
                continue;
            }

            for production in productions
                .iter()
                .filter(|production| production.left == non_terminal)
            {
                let next_item = Item {
                    production: production.clone(),
                    dot_pos: 0,
                };

                if closure_set.insert(next_item.clone()) {
                    new_items_to_add.insert(next_item);
                    added_this_round = true;
                }
            }
        }

        if !added_this_round {
            break;
        }
    }

    closure_set
}

fn goto(
    items: &BTreeSet<Item>,
    symbol: Symbol,
    productions: &[Production],
    non_terminals: &BTreeSet<NonTerminal>,
) -> BTreeSet<Item> {
    let mut next_items = BTreeSet::new();

    for item in items {
        if item.dot_pos < item.production.right.len()
            && item.production.right[item.dot_pos] == symbol
        {
            let mut next_item = item.clone();
            next_item.dot_pos += 1;
            next_items.insert(next_item);
        }
    }

    closure(&next_items, productions, non_terminals)
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
struct Item {
    production: Production,
    dot_pos: usize,
}

pub(crate) type InternalState = usize;
type ProductionId = usize;

pub(crate) struct CompiledParser {
    productions: Vec<Production>,
    action_table: BTreeMap<(InternalState, Terminal), Action>,
    goto_table: BTreeMap<(InternalState, NonTerminal), InternalState>,
    start_state: InternalState,
}

impl CompiledParser {
    pub(crate) fn start_state(&self) -> InternalState {
        self.start_state
    }

    pub(crate) fn action(&self, state: InternalState, terminal: Terminal) -> Option<Action> {
        self.action_table.get(&(state, terminal)).copied()
    }

    pub(crate) fn goto(
        &self,
        state: InternalState,
        non_terminal: NonTerminal,
    ) -> Option<InternalState> {
        self.goto_table.get(&(state, non_terminal)).copied()
    }

    pub(crate) fn production(&self, id: ProductionId) -> Option<&Production> {
        self.productions.get(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Shift(InternalState),
    Reduce(ProductionId),
    Accept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    ConflictReducer,
    MissingProduction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse_grammar_text;

    #[test]
    fn compile_builds_start_state() {
        let grammar = parse_grammar_text("E -> E+B\nE -> B\nB -> 0\nB -> 1").unwrap();
        let machine = compile(&grammar).unwrap();

        assert_eq!(machine.start_state(), 0);
    }
}
