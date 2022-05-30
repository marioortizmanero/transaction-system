use anyhow::Result;

use transaction_system::model::Balance;

fn run_test(input: &str, output: &str) -> Result<()> {
    let test_dir = env!("CARGO_MANIFEST_DIR");
    let input = format!("{test_dir}/tests/{input}");
    let output = format!("{test_dir}/tests/{output}");

    let ret_balances = transaction_system::process(&input)?;

    let mut expected_balances = Balance::init_all();
    let mut reader = transaction_system::init_reader(&output)?;
    for result in reader.deserialize::<Balance>() {
        let entry = result?;
        let client = entry.client as usize;
        expected_balances[client] = Some(entry);
    }

    // Prettier on console by iterating one by one
    let mut i = 1;
    for (ret, expected) in ret_balances.into_iter().zip(expected_balances) {
        assert_eq!(ret, expected, "ret != {output} for client {i}");
        i += 1;
    }

    Ok(())
}

#[test]
fn test_simple() {
    // Making sure both output formats work
    run_test("inputs/simple.csv", "outputs/simple.csv").unwrap();
    run_test("inputs/simple.csv", "outputs/simple_alternate.csv").unwrap();
}

#[test]
fn test_case() {
    // Making case insensitive input is accepted
    run_test("inputs/case.csv", "outputs/case.csv").unwrap();
}

#[test]
fn test_floatingprecision() {
    // Only 4 digits of precision
    run_test(
        "inputs/floatingprecision.csv",
        "outputs/floatingprecision.csv",
    )
    .unwrap();
    run_test(
        "inputs/floatingprecision.csv",
        "outputs/floatingprecision_alternate.csv",
    )
    .unwrap();
}

#[test]
fn test_duplicate() {
    // Repeated transaction IDs should be handled correctly
    run_test("inputs/duplicate1.csv", "outputs/duplicate1.csv").unwrap();
    run_test("inputs/duplicate2.csv", "outputs/duplicate2.csv").unwrap();
    run_test("inputs/duplicate3.csv", "outputs/duplicate3.csv").unwrap();
}

#[test]
fn test_dispute() {
    run_test("inputs/dispute.csv", "outputs/dispute.csv").unwrap();
}

#[test]
fn test_resolve() {
    run_test("inputs/resolve1.csv", "outputs/resolve1.csv").unwrap();
    run_test("inputs/resolve2.csv", "outputs/resolve2.csv").unwrap();
}

#[test]
fn test_chargeback() {
    run_test("inputs/chargeback1.csv", "outputs/chargeback1.csv").unwrap();
    run_test("inputs/chargeback2.csv", "outputs/chargeback2.csv").unwrap();
}

#[test]
fn test_fulldisputes() {
    // Checks the rest of edge cases for deposit, resolve, and chargeback
    run_test("inputs/fulldisputes.csv", "outputs/fulldisputes.csv").unwrap();
}
