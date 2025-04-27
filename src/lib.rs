// Library entry point for cdcl2
pub mod stack_stack;
pub mod variables;
pub mod clauses;

use crate::clauses::{Clause, SingleUnitPropResult};
use stack_stack::StackStack;
use variables::{Atom, VarRef};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
enum DerivationGraphNode {
    Guess { level: usize },
    Derivation { level: usize, inputs: Vec<Atom> },
}

impl DerivationGraphNode {
    fn level(&self) -> usize {
        match self {
            DerivationGraphNode::Guess { level } => *level,
            DerivationGraphNode::Derivation { level, .. } => *level,
        }
    }
}

struct State {
    clauses: Vec<Clause>,
    assignment_stack: StackStack<Atom>,
    assignments: Vec<Option<bool>>,
    derivation_graph: Vec<Option<DerivationGraphNode>>,
}

enum FullUnitPropResult {
    Conflict { inputs: Vec<Atom> },
    Done,
}

impl State {
    fn new(clauses: Vec<Clause>, num_vars: usize) -> Self {
        State {
            clauses,
            assignment_stack: StackStack::new(),
            assignments: vec![None; num_vars],
            derivation_graph: vec![None; num_vars],
        }
    }

    fn unit_prop_scan(&mut self) -> FullUnitPropResult {
        let mut done_something = true;
        while done_something {
            done_something = false;
            for clause in self.clauses.iter_mut() {
                match clause.unit_prop_clause(&self.assignments) {
                    SingleUnitPropResult::Conflict => {
                        return FullUnitPropResult::Conflict {
                            inputs: clause.body.iter().map(|x| x.negated()).collect(),
                        };
                    }
                    SingleUnitPropResult::None => {}
                    SingleUnitPropResult::UnitProp { inputs, output } => {
                        let node = DerivationGraphNode::Derivation {
                            level: self.assignment_stack.len(),
                            inputs,
                        };
                        self.derivation_graph[output.get_var().id] = Some(node);
                        self.assignments[output.get_var().id] = Some(output.polarity());
                        self.assignment_stack.push_value(output);
                        done_something = true;
                    }
                }
            }
        }
        FullUnitPropResult::Done
    }
    fn analyse_conflict(&self, inputs: &[Atom]) -> (Clause, usize) {
        let cur_level = self.assignment_stack.len();
        let mut reset_level = 0;
        let mut result_body = Vec::new();
        let mut frontier = vec![false; self.assignments.len()];
        let mut cur_level_size = 0;
        let mut iter = self.assignment_stack.peek_stack().iter().rev();
        let mut next_conflict = inputs;
        loop {
            for atom in next_conflict {
                let seen_prev: bool = frontier[atom.get_var().id];
                if !seen_prev {
                    frontier[atom.get_var().id] = true;
                    if let Some(node) = &self.derivation_graph[atom.get_var().id] {
                        if node.level() == cur_level {
                            cur_level_size += 1;
                        } else if node.level() != 0 {
                            reset_level = reset_level.max(node.level());
                            result_body.push(atom.negated());
                        }
                    } else {
                        panic!("Node not found in derivation graph, but was in conflict");
                    }
                }
            }
            let next_frontier = loop {
                let Some(atom) = iter.next() else {
                    panic!("Could not find a UIP")
                };
                if frontier[atom.get_var().id] {
                    frontier[atom.get_var().id] = false;
                    cur_level_size -= 1;
                    break atom;
                }
            };
            if cur_level_size == 0 {
                result_body.push(next_frontier.negated());
                return (Clause::new(result_body), reset_level);
            }
            match &self.derivation_graph[next_frontier.get_var().id] {
                Some(DerivationGraphNode::Guess { .. }) => {
                    panic!("Found the guess node, but expected it to be a UIP");
                }
                None => {
                    panic!("Node not found in derivation graph, but was in conflict");
                }
                Some(DerivationGraphNode::Derivation { inputs, .. }) => {
                    next_conflict = inputs;
                }
            }
        }
    }
    fn guess(&self) -> Option<Atom> {
        for (i, assignment) in self.assignments.iter().enumerate() {
            if assignment.is_none() {
                return Some(VarRef { id: i }.as_atom(true));
            }
        }
        None
    }
    fn make_guess(&mut self, guess: Atom) {
        let node = DerivationGraphNode::Guess {
            level: self.assignment_stack.len() + 1,
        };
        self.derivation_graph[guess.get_var().id] = Some(node);
        self.assignments[guess.get_var().id] = Some(guess.polarity());
        self.assignment_stack.push_stack();
        self.assignment_stack.push_value(guess);
    }
    fn sat(&mut self) -> bool {
        loop {
            match self.unit_prop_scan() {
                FullUnitPropResult::Conflict { inputs } => {
                    if self.assignment_stack.len() == 0 {
                        return false;
                    }
                    let (conflict_clause, reset_level) = self.analyse_conflict(&inputs);
                    self.clauses.push(conflict_clause);
                    while self.assignment_stack.len() > reset_level {
                        for var in self.assignment_stack.pop_stack_iter() {
                            self.assignments[var.get_var().id] = None;
                            self.derivation_graph[var.get_var().id] = None;
                        }
                    }
                }
                FullUnitPropResult::Done => {
                    let guess = self.guess();
                    if let Some(guess) = guess {
                        self.make_guess(guess);
                    } else {
                        return true;
                    }
                }
            }
        }
    }
}

pub fn run_cnf(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let mut var_manager: HashMap<String, VarRef> = HashMap::new();
    let mut max_var = 0;
    let mut clauses = Vec::new();
    for line in input.lines() {
        let line = line?;
        let mut atoms = Vec::new();
        for atom in line.split_whitespace() {
            let (name, polarity) = if atom.starts_with('!') {
                (atom[1..].to_string(), false)
            } else {
                (atom.to_string(), true)
            };
            let var_ref = var_manager.entry(name).or_insert_with(|| {
                let var = max_var;
                max_var += 1;
                VarRef { id: var }
            });
            atoms.push(var_ref.as_atom(polarity));
        }
        if atoms.is_empty() {
            break;
        }
        clauses.push(Clause::new(atoms));
    }
    let mut state = State::new(clauses, var_manager.len());
    let sat = state.sat();
    if sat {
        writeln!(output, "SAT")?;
        for var in var_manager.iter() {
            let val = match state.assignments[var.1.id] {
                Some(true) => "1",
                Some(false) => "0",
                None => "_",
            };
            writeln!(output, "{} {}", var.0, val)?;
        }
    } else {
        writeln!(output, "UNSAT")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clauses::Clause;
    use crate::variables::VarRef;

    #[test]
    fn test_analyse_conflict_realistic() {
        // Clauses: (!a, b), (!a, !b)
        // Assign a = true at level 1 (decision)
        // Derive !b at level 1 from (!a, !b) and a = true
        // Conflict between !a and !b
        let mut state = State::new(
            vec![
                Clause::new(vec![VarRef { id: 0 }.as_atom(false), VarRef { id: 1 }.as_atom(true)]),  // (!a, b)
                Clause::new(vec![VarRef { id: 0 }.as_atom(false), VarRef { id: 1 }.as_atom(false)]), // (!a, !b)
            ],
            2,
        );
        // Level 1: a = true (decision)
        state.assignment_stack.push_stack();
        state.assignment_stack.push_value(VarRef { id: 0 }.as_atom(true));
        state.assignments[0] = Some(true);
        state.derivation_graph[0] = Some(DerivationGraphNode::Guess { level: 1 });
        // Derive !b at level 1 from (!a, !b)
        state.assignment_stack.push_value(VarRef { id: 1 }.as_atom(false));
        state.assignments[1] = Some(false);
        state.derivation_graph[1] = Some(DerivationGraphNode::Derivation {
            level: 1,
            inputs: vec![VarRef { id: 0 }.as_atom(false)],
        });
        // Conflict between !a and !b (i.e., both required to be true)
        let inputs = &[VarRef { id: 0 }.as_atom(false), VarRef { id: 1 }.as_atom(false)];
        let (conflict_clause, reset_level) = state.analyse_conflict(inputs);
        // The learned clause should be [!a]
        assert_eq!(conflict_clause.body, vec![VarRef { id: 0 }.as_atom(false)]);
        assert_eq!(reset_level, 0);
    }
}
