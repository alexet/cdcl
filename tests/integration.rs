// Integration tests for the CDCL SAT solver
// Place in tests/ directory for cargo to pick up
use std::io::Cursor;
use cdcl2::run_cnf;

#[test]
fn test_simple_sat() {
    let input = b"a b\n!a\n";
    let cursor = Cursor::new(&input[..]);
    let mut output = Vec::new();
    let res = run_cnf(cursor, &mut output);
    assert!(res.is_ok());
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("SAT"));
}

#[test]
fn test_unsat() {
    let input = b"a\n!a\n";
    let cursor = Cursor::new(&input[..]);
    let mut output = Vec::new();
    let res = run_cnf(cursor, &mut output);
    assert!(res.is_ok());
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("UNSAT"));
}

#[test]
fn test_trivial_sat() {
    let input = b"a\n\n";
    let cursor = Cursor::new(&input[..]);
    let mut output = Vec::new();
    let res = run_cnf(cursor, &mut output);
    assert!(res.is_ok());
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("SAT"));
}

#[test]
fn test_conflict_with_two_clauses() {
    // Clauses: !a b and !a !b
    let input = b"!a b\n!a !b\n\n";
    let cursor = Cursor::new(&input[..]);
    let mut output = Vec::new();
    let res = run_cnf(cursor, &mut output);
    assert!(res.is_ok());
    let output_str = String::from_utf8(output).unwrap();
    // The only way to satisfy both is a = false, b can be anything
    // But if a = true, both clauses are unit and force b and !b, so UNSAT
    // The solver should find SAT (a = false)
    assert!(output_str.contains("SAT"));
    assert!(output_str.contains("a 0"));
}
