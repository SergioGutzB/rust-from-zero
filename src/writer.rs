use std::fs::File;
use std::io::{self, Write};

use crate::error::AppError;
use crate::parser::{EXPECTED_HEADER, Sale};

/// Demonstrates shared read-only access (`&[T]`).
/// Writing only needs to read each record, so borrowing lets the caller keep
/// ownership and reuse the data after writing (e.g. write to stdout AND to a
/// file without cloning).
///
/// Generic over `W: Write` so the same code serves stdout, files, and the
/// in-memory buffers used by tests.
fn write_records<W: Write>(mut target: W, records: &[Sale]) -> Result<(), AppError> {
    writeln!(target, "{}", EXPECTED_HEADER)?;

    for record in records {
        writeln!(
            target,
            "{},{},{},{},{}",
            record.id, record.name, record.region, record.amount, record.active
        )?;
    }

    target.flush()?;

    Ok(())
}

pub fn write_to_stdout(records: &[Sale]) -> Result<(), AppError> {
    let stdout = io::stdout();
    let handle = stdout.lock();

    write_records(handle, records)
}

pub fn write_to_file(file_path: &str, records: &[Sale]) -> Result<(), AppError> {
    let file = File::create(file_path)?;

    write_records(file, records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_records_success() {
        let records = vec![
            Sale::new(1, "Mouse".to_string(), "EU".to_string(), 25.50, true),
            Sale::new(2, "Keyboard".to_string(), "EU".to_string(), 79.99, true),
        ];

        let mut buffer = Vec::new();
        let result = write_records(&mut buffer, &records);
        assert!(result.is_ok());

        let output_text = String::from_utf8(buffer).expect("output is valid UTF-8");

        let expected_output =
            "id,name,region,amount,active\n1,Mouse,EU,25.5,true\n2,Keyboard,EU,79.99,true\n";
        assert_eq!(output_text, expected_output);
    }

    #[test]
    fn test_write_records_empty_writes_header_only() {
        let records: Vec<Sale> = vec![];

        let mut buffer = Vec::new();
        write_records(&mut buffer, &records).expect("writing to a Vec cannot fail");

        let output_text = String::from_utf8(buffer).expect("output is valid UTF-8");
        assert_eq!(output_text, "id,name,region,amount,active\n");
    }

    #[test]
    fn test_write_to_file_empty_path_error() {
        let records = vec![Sale::new(
            1,
            "Mouse".to_string(),
            "EU".to_string(),
            25.50,
            true,
        )];

        let result = write_to_file("", &records);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::IoError(_)));
    }

    #[test]
    fn test_write_to_file_permission_error() {
        let records = vec![Sale::new(
            1,
            "Mouse".to_string(),
            "EU".to_string(),
            25.50,
            true,
        )];

        let result = write_to_file("/etc/output.csv", &records);

        assert!(result.is_err());
    }
}
