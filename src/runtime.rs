use crate::ast::AstNode;
use crate::grammar::Symbol;
use crate::lr::{Action, CompiledParser, InternalState};

#[derive(Debug, Clone)]
pub enum StepAction {
    Shift { terminal: char, to_state: usize },
    Reduce { rule: String, pop_count: usize },
    Accept,
}

#[derive(Debug, Clone)]
pub struct ParseStep {
    pub action: StepAction,
    pub from_state: usize,
    pub lookahead: char,
    pub state_stack: Vec<usize>,
    pub remaining_input: Vec<char>,
    pub ast_stack: Vec<AstNode>,
}

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

pub fn build_trace(
    machine: &CompiledParser,
    input: &[Symbol],
) -> Result<Vec<ParseStep>, RuntimeError> {
    let mut steps = Vec::new();
    let mut state = ParserState::new(input.to_vec(), machine.start_state());

    loop {
        let from_state = state.current_state()?;
        let next_sym = state
            .remaining_input
            .first()
            .cloned()
            .ok_or(RuntimeError::ExpectedTerminalInput)?;
        let Symbol::Terminal(terminal) = next_sym else {
            return Err(RuntimeError::ExpectedTerminalInput);
        };
        let lookahead = terminal.0;

        let action = machine
            .action(from_state, terminal)
            .ok_or(RuntimeError::InvalidAction)?;

        let step_action = match &action {
            Action::Shift(to) => StepAction::Shift { terminal: lookahead, to_state: *to },
            Action::Reduce(prod_id) => {
                let prod = machine
                    .production(*prod_id)
                    .ok_or(RuntimeError::InvalidReduce)?;
                let rhs: String = prod
                    .right
                    .iter()
                    .map(|s| match s {
                        Symbol::Terminal(t) => t.0.to_string(),
                        Symbol::NonTerminal(nt) => nt.0.to_string(),
                    })
                    .collect();
                StepAction::Reduce {
                    rule: format!("{} -> {}", prod.left.0, rhs),
                    pop_count: prod.right.len(),
                }
            }
            Action::Accept => StepAction::Accept,
        };

        let step_result = step(machine, state)?;

        match step_result {
            StepResult::Continue(next) => {
                let remaining: Vec<char> = next
                    .remaining_input
                    .iter()
                    .filter_map(|s| {
                        if let Symbol::Terminal(t) = s {
                            Some(t.0)
                        } else {
                            None
                        }
                    })
                    .collect();
                steps.push(ParseStep {
                    action: step_action,
                    from_state,
                    lookahead,
                    state_stack: next.state_stack.clone(),
                    remaining_input: remaining,
                    ast_stack: next.ast_stack.clone(),
                });
                state = next;
            }
            StepResult::Accept(result) => {
                steps.push(ParseStep {
                    action: step_action,
                    from_state,
                    lookahead,
                    state_stack: vec![],
                    remaining_input: vec![],
                    ast_stack: vec![result.ast.clone()],
                });
                break;
            }
        }
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{Symbol, Terminal, parse_grammar_text};
    use crate::lr::compile;

    #[test]
    fn dump_trace_for_debug() {
        let grammar = parse_grammar_text("E -> E*B\nE -> E+B\nE -> B\nB -> 0\nB -> 1").unwrap();
        let machine = compile(&grammar).unwrap();
        let input = [
            Symbol::Terminal(Terminal('1')),
            Symbol::Terminal(Terminal('+')),
            Symbol::Terminal(Terminal('1')),
            Symbol::Terminal(Terminal('*')),
            Symbol::Terminal(Terminal('1')),
            Symbol::Terminal(Terminal('$')),
        ];

        let trace = build_trace(&machine, &input).unwrap();

        println!("\n=== PARSE TRACE DUMP ===");
        println!("grammar: E->E*B | E->E+B | E->B | B->0 | B->1");
        println!("input:   1+1*1");
        println!("steps:   {}", trace.len());
        println!();

        for (i, step) in trace.iter().enumerate() {
            let action_str = match &step.action {
                StepAction::Shift { terminal, to_state } =>
                    format!("SHIFT '{}' -> state {}", terminal, to_state),
                StepAction::Reduce { rule, pop_count } =>
                    format!("REDUCE {} (pop {})", rule, pop_count),
                StepAction::Accept => "ACCEPT".to_string(),
            };

            let sm_edge = match &step.action {
                StepAction::Shift { terminal, to_state } =>
                    format!("sm_edge: {} --'{}'-> {}", step.from_state, terminal, to_state),
                StepAction::Reduce { rule, .. } =>
                    format!("sm_edge: <none; reduce {}>", rule),
                StepAction::Accept =>
                    format!("sm_edge: <none; accept at {}>", step.from_state),
            };

            let stack: String = step.state_stack.iter()
                .map(|s| s.to_string()).collect::<Vec<_>>().join(",");
            let remaining: String = step.remaining_input.iter().collect();

            println!("step {:>2}: [pre]  from_state={}  lookahead='{}'  action={}",
                i + 1, step.from_state, step.lookahead, action_str);
            println!("         [post] stack=[{}]  remaining='{}'", stack, remaining);
            println!("         [sm]   {}", sm_edge);
            println!();
        }
        println!("=== END TRACE DUMP ===");
    }

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
