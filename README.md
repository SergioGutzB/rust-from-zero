# parse-lib

A command-line data-processing tool built in Rust as part of a specialization course.

Reads a CSV dataset, transforms it with lazy iterator pipelines and closures, and writes the result to stdout or a file.

## Usage

```sh
cargo run -- <file_path> <command> [args] [--output <path>]
```

Commands:

| Command           | Description                                               |
| ----------------- | --------------------------------------------------------- |
| `filter <region>` | Keep active sales for `<region>`, sorted by amount (desc) |
| `map`             | Uppercase every sale name                                 |
| `aggregate`       | Print the total revenue of active sales                   |

Examples:

```sh
cargo run -- datos.csv filter EU
cargo run -- datos.csv map --output mapped.csv
cargo run -- datos.csv aggregate
```

Status messages go to stderr; stdout carries only data, so output is pipeable.

## Features

- CSV input/output with header validation
- Filter, map, and aggregate transformations selected from the CLI
- Lazy iterator chains with a single materialization point
- Custom error enum (`AppError`) with `From` impls and meaningful context
- Documented ownership patterns: `&[T]`, `&mut [T]`, by-value `Vec<T>`, and a generic `summarize<T: Into<f64>>`

## Project Structure

- `src/lib.rs` — module wiring and `pub use` re-exports
- `src/error.rs` — `AppError` enum and `From` conversions
- `src/parser.rs` — `Sale` struct, line parsing, lazy file reading
- `src/transform.rs` — filter / map / aggregate transformations
- `src/writer.rs` — CSV output to stdout or file
- `src/main.rs` — CLI entrypoint and argument parsing
- `tests/` — end-to-end tests against `tests/fixtures/sample.csv`

## Tests

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```
