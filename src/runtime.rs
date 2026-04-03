use crate::ast::AstNode;
use crate::grammar::Symbol;
use crate::lr::{Action, CompiledParser, InternalState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserResult {
    pub ast: AstNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Continue(ParserState),
    Accept(ParserResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    EmptyStateStack,
    ExpectedTerminalInput,
    InvalidAction,
    InvalidReduce,
    MissingGoto,
    MissingAst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserState {
    pub state_stack: Vec<InternalState>,
    pub ast_stack: Vec<AstNode>,
    pub remaining_input: Vec<Symbol>,
}

impl ParserState {
    pub fn new(input: Vec<Symbol>, start_state: InternalState) -> Self {
        Self {
            state_stack: vec![start_state],
            ast_stack: Vec::new(),
            remaining_input: input,
        }
    }

    fn current_state(&self) -> Result<InternalState, RuntimeError> {
        self.state_stack
            .last()
            .copied()
            .ok_or(RuntimeError::EmptyStateStack)
    }
}

pub fn step(
    machine: &CompiledParser,
    state: ParserState,
) -> Result<StepResult, RuntimeError> {
    let current_state = state.current_state()?;
    let next_symbol = state
        .remaining_input
        .first()
        .cloned()
        .ok_or(RuntimeError::ExpectedTerminalInput)?;

    let Symbol::Terminal(terminal) = next_symbol.clone() else {
        return Err(RuntimeError::ExpectedTerminalInput);
    };

    let action = machine
        .action(current_state, terminal)
        .ok_or(RuntimeError::InvalidAction)?;

    match action {
        Action::Shift(next_state) => {
            let mut next = state;
            next.state_stack.push(next_state);
            next.ast_stack.push(AstNode::Terminal(terminal.0));
            next.remaining_input.remove(0);
            Ok(StepResult::Continue(next))
        }
        Action::Reduce(production_id) => {
            let production = machine
                .production(production_id)
                .ok_or(RuntimeError::InvalidReduce)?
                .clone();

            let pop_count = production.right.len();
            if state.state_stack.len() < pop_count {
                return Err(RuntimeError::InvalidReduce);
            }
            if state.ast_stack.len() < pop_count {
                return Err(RuntimeError::MissingAst);
            }

            let mut next = state;
            let children = next
                .ast_stack
                .drain(next.ast_stack.len() - pop_count..)
                .collect();
            next.state_stack
                .truncate(next.state_stack.len() - pop_count);

            let goto_from = next.current_state()?;
            let goto_state = machine
                .goto(goto_from, production.left)
                .ok_or(RuntimeError::MissingGoto)?;

            next.ast_stack
                .push(AstNode::NonTerminal(production.left.0, children));
            next.state_stack.push(goto_state);

            Ok(StepResult::Continue(next))
        }
        Action::Accept => {
            let ast = state
                .ast_stack
                .last()
                .cloned()
                .ok_or(RuntimeError::MissingAst)?;
            Ok(StepResult::Accept(ParserResult { ast }))
        }
    }
}

pub fn run(
    machine: &CompiledParser,
    input: &[Symbol],
) -> Result<ParserResult, RuntimeError> {
    let mut state = ParserState::new(input.to_vec(), machine.start_state());

    loop {
        match step(machine, state)? {
            StepResult::Continue(next_state) => state = next_state,
            StepResult::Accept(result) => return Ok(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{Symbol, Terminal, parse_grammar_text};
    use crate::lr::compile;

    #[test]
    fn run_returns_an_ast() {
        let grammar = parse_grammar_text("E -> E+B\nE -> B\nB -> 0\nB -> 1").unwrap();
        let machine = compile(&grammar).unwrap();
        let input = [
            Symbol::Terminal(Terminal('1')),
            Symbol::Terminal(Terminal('+')),
            Symbol::Terminal(Terminal('0')),
            Symbol::Terminal(Terminal('$')),
        ];

        let result = run(&machine, &input).unwrap();

        assert_eq!(
            result.ast,
            AstNode::NonTerminal(
                'E',
                vec![
                    AstNode::NonTerminal(
                        'E',
                        vec![AstNode::NonTerminal('B', vec![AstNode::Terminal('1')])],
                    ),
                    AstNode::Terminal('+'),
                    AstNode::NonTerminal('B', vec![AstNode::Terminal('0')]),
                ],
            )
        );
    }
}
