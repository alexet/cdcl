use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    io::{self, stdin, BufRead},
};

use crate::{
    stack_stack::StackStack,
    variables::{Atom, VarRef},
};

mod stack_stack;
mod variables;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Clause {
    body: Vec<Atom>,
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for atom in &self.body {
            write!(f, "{} ", atom)?;
        }
        Ok(())
    }
}

enum SingleUnitPropResult {
    Conflict,
    None,
    UnitProp { inputs: Vec<Atom>, output: Atom },
}

impl Clause {
    fn new(body: Vec<Atom>) -> Self {
        Clause { body }
    }

    // Unit propagate a single clause
    fn unit_prop_clause(&mut self, assignments: &[Option<bool>]) -> SingleUnitPropResult {
        // The single unassigned non-conflicting variable
        // Or none if all variables conflict
        let mut unit_result: Option<Atom> = Option::None;

        for atom in &self.body {
            // Check if the atom is assigned
            match &assignments[atom.get_var().id] {
                Some(assignment) => {
                    // It is assigneed
                    if atom.polarity() == *assignment {
                        // Clause is true
                        // Nothing useful
                        return SingleUnitPropResult::None;
                    } else {
                        // This assignment conflicts
                        // Keep looking
                    }
                }
                None => {
                    if let Some(_) = unit_result {
                        // 2 unassigned atoms
                        // Can't do much useful
                        return SingleUnitPropResult::None;
                    } else {
                        // Otherwise this is the only unassigned variable
                        unit_result = Some(*atom);
                    }
                }
            }
        }

        if let Some(unit) = unit_result {
            // Single unassigned variable
            // Create the unit prop result
            let mut atom_deps: Vec<Atom> = Vec::new();
            for atom in &self.body {
                match &assignments[atom.get_var().id] {
                    Some(assignment) => {
                        if atom.polarity() != *assignment {
                            atom_deps.push(atom.negated());
                        }
                    }
                    None => {}
                }
            }
            SingleUnitPropResult::UnitProp {
                inputs: atom_deps,
                output: unit,
            }
        } else {
            // All atoms are assigned and none are true
            SingleUnitPropResult::Conflict
        }
    }
}

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
        // Scan all clauses and unit propagate
        // until no more unit propagation is possible
        // or until a conflict is found
        // We will use a flag to indicate if we have done something
        // in the last iteration and wait until we have done nothing
        // to stop
        let mut done_something = true;
        while done_something {
            done_something = false;
            for clause in self.clauses.iter_mut() {
                match clause.unit_prop_clause(&self.assignments) {
                    SingleUnitPropResult::Conflict => {
                        // Report the conflict
                        return FullUnitPropResult::Conflict {
                            inputs: clause.body.iter().map(|x| x.negated()).collect(),
                        };
                    }
                    SingleUnitPropResult::None => {
                        // Nothing to do
                    }
                    SingleUnitPropResult::UnitProp { inputs, output } => {
                        let node = DerivationGraphNode::Derivation {
                            level: self.assignment_stack.len(),
                            inputs,
                        };
                        self.derivation_graph[output.get_var().id] = Some(node);
                        self.assignments[output.get_var().id] = Some(output.polarity());
                        self.assignment_stack.push_value(output);
                        // Ensure we traverse everything a second time.
                        done_something = true;
                    }
                }
            }
        }
        // If we scanned all clauses and did not find anything
        // we are done
        FullUnitPropResult::Done
    }

    /// Analyse the conflict, we find a UIP
    fn analyse_conflict(&self, inputs: &[Atom]) -> (Clause, usize) {
        // The level of the current assignment stack
        // Note that we expect the conflict to derive from
        // the guess at the current level
        let cur_level = self.assignment_stack.len();

        // The highest level of the frontier excluding the current level
        let mut reset_level = 0;
        // The nodes in previous levels of the frontier
        // This will become the body of the new clause
        let mut result_body = Vec::new();

        // The frontier is a vector of booleans
        // each index corresponds to a variable and true
        // means that the variable is in the frontier
        let mut frontier = vec![false; self.assignments.len()];

        // The number of frontier variables in the current level
        let mut cur_level_size = 0;

        // The iterator over the assignments backwards
        let mut iter = self.assignment_stack.peek_stack().iter().rev();

        // The next conflict to resolve
        let mut next_conflict = inputs;

        // Invariants:
        // The frontier assignments + the next_conflict atoms + level 0 nodes are a conflict.
        // The next_conflict variables all are assigned earlier than the iterator position
        // The number of frontier variables in the current level is cur_level_size
        // All frontier variables not in the current level are in result_body
        // The reset_level is the highest level of the result_body
        loop {
            // Start by moving the next conflict to the frontier
            for atom in next_conflict {
                let seen_prev: bool = frontier[atom.get_var().id];
                if !seen_prev {
                    frontier[atom.get_var().id] = true;
                    if let Some(node) = &self.derivation_graph[atom.get_var().id] {
                        if node.level() == cur_level {
                            cur_level_size += 1;
                        } else if node.level() != 0 {
                            // This node contributes to the
                            reset_level = reset_level.max(node.level());
                            result_body.push(atom.negated());
                        } else {
                            // This is a level 0 node
                            // so it is does not need to be added
                        }
                    } else {
                        panic!("Node not found in derivation graph, but was in conflict");
                    }
                }
            }

            // Find the next varible to simplify with
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
                // This is the UIP
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
                    // We have found the next conflict to resolve.
                    next_conflict = inputs;
                }
            }
        }
    }

    // A silly guess function for now
    fn guess(&self) -> Option<Atom> {
        // Guess the next variable to assign
        // We will use the first unassigned variable
        for (i, assignment) in self.assignments.iter().enumerate() {
            if assignment.is_none() {
                return Some(VarRef { id: i }.as_atom(true));
            }
        }
        None
    }

    fn make_guess(&mut self, guess: Atom) {
        // Make a guess
        // We will use the first unassigned variable
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
                        // We are done (UNSAT)
                        return false;
                    }
                    let (conflict_clause, reset_level) = self.analyse_conflict(&inputs);
                    self.clauses.push(conflict_clause);
                    // Reset the assignment stack to the reset level
                    // We will pop the stack until we reach the reset level
                    while self.assignment_stack.len() > reset_level {
                        for var in self.assignment_stack.pop_stack_iter() {
                            // Remove the variable from the assignment
                            self.assignments[var.get_var().id] = None;
                            // Remove the variable from the derivation graph
                            self.derivation_graph[var.get_var().id] = None;
                        }
                    }
                }
                FullUnitPropResult::Done => {
                    let guess = self.guess();
                    if let Some(guess) = guess {
                        self.make_guess(guess);
                    } else {
                        // We are done (SAT)
                        return true;
                    }
                }
            }
        }
    }
}

fn run_cnf(input : impl BufRead) -> io::Result<()> {
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
            // Terminate on empty line
            break;
        }
        clauses.push(Clause::new(atoms));
    }

    let mut state = State::new(clauses, var_manager.len());
    let sat = state.sat();
    if sat {
        println!("SAT");
        for var in var_manager.iter() {
            let val = match state.assignments[var.1.id] {
                Some(true) => "1",
                Some(false) => "0",
                None => "_",
            };
            println!("{} {}", var.0, val);
        }
    } else {
        println!("UNSAT");
    }
    Ok(())
}

fn main() -> io::Result<()> {
    run_cnf(stdin().lock())
}