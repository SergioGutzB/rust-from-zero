use std::process::{Command, Output};

const FIXTURE: &str = "tests/fixtures/sample.csv";

/// Runs the compiled binary (resolved by Cargo at build time) with the given
/// arguments. Avoids nesting `cargo run` inside `cargo test`, which is slower
/// and can interleave compilation output with the assertions.
fn run_binary(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_parse-lib"))
        .args(args)
        .output()
        .expect("binary built by cargo must be executable")
}

#[test]
fn test_cli_filter_success() {
    let output = run_binary(&[FIXTURE, "filter", "EU"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Parsed 7 records"));
    assert!(stdout.contains("id,name,region,amount,active"));
    // Active EU sales sorted by amount desc: Alice (1200.50) first.
    assert!(stdout.contains("1,Alice,EU,1200.5,true"));
    // Inactive (Elena) and other regions (Bob/US) must be excluded.
    assert!(!stdout.contains("Elena"));
    assert!(!stdout.contains("Bob"));

    let first_record = stdout.lines().nth(1).expect("at least one record");
    assert!(first_record.starts_with("1,Alice"));
}

#[test]
fn test_cli_map_success() {
    let output = run_binary(&[FIXTURE, "map"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1,ALICE,EU,1200.5,true"));
    assert!(stdout.contains("3,CAROL,APAC,0,false"));
}

#[test]
fn test_cli_aggregate_success() {
    let output = run_binary(&[FIXTURE, "aggregate"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 1200.50 + 850.00 + 99.99 + 310.75 + 45.10 (active rows only)
    assert_eq!(stdout.trim(), "total_active_revenue,2506.34");
}

#[test]
fn test_cli_output_flag_writes_file() {
    let out_path = std::env::temp_dir().join("parse_lib_e2e_output.csv");
    let out_path_str = out_path.to_str().expect("temp path is valid UTF-8");

    let output = run_binary(&[FIXTURE, "filter", "US", "--output", out_path_str]);

    assert!(output.status.success());

    let content = std::fs::read_to_string(&out_path).expect("output file must exist");
    assert!(content.contains("2,Bob,US,850,true"));
    assert!(!content.contains("Alice"));
}

#[test]
fn test_cli_missing_file_fails() {
    let output = run_binary(&["/nonexistent/data.csv", "filter", "EU"]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Application error"));
    assert!(stderr.contains("IO error"));
}

#[test]
fn test_cli_missing_arguments_fails() {
    let output = run_binary(&[]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage"));
}

#[test]
fn test_cli_unknown_command_fails() {
    let output = run_binary(&[FIXTURE, "explode"]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown command 'explode'"));
}
