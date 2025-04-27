# CDCL: A Simple CDCL SAT Solver in Rust

This repository contains an educational implementation of a SAT solver using the Conflict-Driven Clause Learning (CDCL) algorithm, written in Rust.

## Usage
Build and run the solver with Rust:

```bash
cargo run < input.cnf
```
- Input should be a CNF formula, one clause per line, variables separated by spaces, negation with `!` (e.g., `a !b c`).
- An empty line ends the input.

## Example Input
```
a b
!a c
!b !c
```

## Output
- `SAT` or `UNSAT`
- If SAT, prints each variable and its assignment (1 for true, 0 for false)

## License
MIT License
