use crate::variables::Atom;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Clause {
    pub body: Vec<Atom>,
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for atom in &self.body {
            write!(f, "{} ", atom)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum SingleUnitPropResult {
    Conflict,
    None,
    UnitProp { inputs: Vec<Atom>, output: Atom },
}

impl Clause {
    pub fn new(body: Vec<Atom>) -> Self {
        Clause { body }
    }

    pub fn unit_prop_clause(&mut self, assignments: &[Option<bool>]) -> SingleUnitPropResult {
        let mut unit_result: Option<Atom> = Option::None;
        for atom in &self.body {
            match &assignments[atom.get_var().id] {
                Some(assignment) => {
                    if atom.polarity() == *assignment {
                        return SingleUnitPropResult::None;
                    }
                }
                None => {
                    if let Some(_) = unit_result {
                        return SingleUnitPropResult::None;
                    } else {
                        unit_result = Some(*atom);
                    }
                }
            }
        }
        if let Some(unit) = unit_result {
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
            SingleUnitPropResult::Conflict
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::VarRef;

    #[test]
    fn test_unit_propagation_simple() {
        // Clause: a
        let mut clause = Clause::new(vec![VarRef { id: 0 }.as_atom(true)]);
        let assignments = vec![None];
        match clause.unit_prop_clause(&assignments) {
            SingleUnitPropResult::UnitProp { output, .. } => {
                assert_eq!(output, VarRef { id: 0 }.as_atom(true));
            }
            _ => panic!("Expected unit propagation"),
        }
    }

    #[test]
    fn test_unit_propagation_conflict() {
        // Clause: a, assignment: a = false
        let mut clause = Clause::new(vec![VarRef { id: 0 }.as_atom(true)]);
        let assignments = vec![Some(false)];
        match clause.unit_prop_clause(&assignments) {
            SingleUnitPropResult::Conflict => {},
            _ => panic!("Expected conflict"),
        }
    }

    #[test]
    fn test_unit_propagation_none() {
        // Clause: a b, assignments: a = false, b = true
        let mut clause = Clause::new(vec![VarRef { id: 0 }.as_atom(true), VarRef { id: 1 }.as_atom(true)]);
        let assignments = vec![Some(false), Some(true)];
        match clause.unit_prop_clause(&assignments) {
            SingleUnitPropResult::None => {},
            _ => panic!("Expected no propagation"),
        }
    }
}
