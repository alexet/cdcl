use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarRef {
    pub id: usize,
}

impl VarRef {
    pub fn as_atom(self, polarity: bool) -> Atom {
        Atom {
            var: self,
            polarity,
        }
    }
}

impl Display for VarRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Atom {
    var: VarRef,
    polarity: bool,
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.polarity {
            write!(f, "{}", self.var)
        } else {
            write!(f, "!{}", self.var)
        }
    }
}

impl Atom {
    pub fn negated(&self) -> Atom {
        Atom {
            var: self.var,
            polarity: !self.polarity,
        }
    }

    pub fn polarity(&self) -> bool {
        self.polarity
    }

    pub fn get_var(&self) -> VarRef {
        self.var
    }
}
