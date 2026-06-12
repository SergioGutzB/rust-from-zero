use std::{env, process};

use parse_lib::{AppError, Sale, parser, transform, writer};

const USAGE: &str = "\
Usage: parse-lib <file_path> <command> [args] [--output <path>]

Commands:
  filter <region>   keep active sales for <region>, sorted by amount (desc)
  map               uppercase every sale name
  aggregate         print the total revenue of active sales

Options:
  --output <path>   write the result to <path> instead of stdout";

enum Command {
    Filter(String),
    Map,
    Aggregate,
}

struct Config {
    file_path: String,
    command: Command,
    output: Option<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}

/// Parses CLI arguments into a validated `Config`.
/// Any missing or unknown argument is a recoverable `AppError::Usage`, so the
/// process exits with a non-zero status instead of silently succeeding.
fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Config, AppError> {
    let _binary_name = args.next();

    let file_path = args
        .next()
        .ok_or_else(|| AppError::Usage(format!("missing <file_path>\n\n{USAGE}")))?;

    let command_name = args
        .next()
        .ok_or_else(|| AppError::Usage(format!("missing <command>\n\n{USAGE}")))?;

    let command = match command_name.as_str() {
        "filter" => {
            let region = args.next().ok_or_else(|| {
                AppError::Usage(format!("'filter' requires a <region>\n\n{USAGE}"))
            })?;
            Command::Filter(region)
        }
        "map" => Command::Map,
        "aggregate" => Command::Aggregate,
        other => {
            return Err(AppError::Usage(format!(
                "unknown command '{other}'\n\n{USAGE}"
            )));
        }
    };

    let output =
        match args.next() {
            Some(flag) if flag == "--output" => Some(args.next().ok_or_else(|| {
                AppError::Usage(format!("'--output' requires a <path>\n\n{USAGE}"))
            })?),
            Some(other) => {
                return Err(AppError::Usage(format!(
                    "unexpected argument '{other}'\n\n{USAGE}"
                )));
            }
            None => None,
        };

    Ok(Config {
        file_path,
        command,
        output,
    })
}

fn run() -> Result<(), AppError> {
    let config = parse_args(env::args())?;

    // Single materialization point: the parsing iterator stays lazy until
    // here, and `collect` into `Result<Vec<_>, _>` stops at the first error.
    let sales: Vec<Sale> = parser::read_file(&config.file_path)?.collect::<Result<_, _>>()?;

    // Status goes to stderr so stdout carries only data and stays pipeable.
    eprintln!("Parsed {} records from {}", sales.len(), config.file_path);

    match config.command {
        Command::Filter(region) => {
            let mut result = transform::filter_active_by_region(sales, &region);
            transform::sort_sales_by_amount_desc(&mut result);
            write_records(&config.output, &result)
        }
        Command::Map => {
            let result = transform::uppercase_names(sales);
            write_records(&config.output, &result)
        }
        Command::Aggregate => {
            let total = transform::calculate_total_revenue(&sales);
            write_line(&config.output, &format!("total_active_revenue,{total:.2}"))
        }
    }
}

fn write_records(output: &Option<String>, records: &[Sale]) -> Result<(), AppError> {
    match output {
        Some(path) => writer::write_to_file(path, records),
        None => writer::write_to_stdout(records),
    }
}

fn write_line(output: &Option<String>, line: &str) -> Result<(), AppError> {
    match output {
        Some(path) => Ok(std::fs::write(path, format!("{line}\n"))?),
        None => {
            println!("{line}");
            Ok(())
        }
    }
}
