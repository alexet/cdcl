fn main() -> std::io::Result<()> {
    cdcl2::run_cnf(std::io::stdin().lock(), std::io::stdout().lock())
}