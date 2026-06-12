use crate::error::AppError;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Expected CSV header. The first line of every input file must match it
/// exactly so that a missing header (or a wrong file) fails loudly instead
/// of silently dropping the first data row.
pub const EXPECTED_HEADER: &str = "id,name,region,amount,active";

/// The core data structure representing a single row in the dataset.
#[derive(Debug)]
pub struct Sale {
    pub id: u32,
    pub name: String,
    pub region: String,
    pub amount: f64,
    pub active: bool,
}

impl Sale {
    pub fn new(id: u32, name: String, region: String, amount: f64, active: bool) -> Sale {
        Sale {
            id,
            name,
            region,
            amount,
            active,
        }
    }
}

/// Parses a single comma-separated line into a Sale struct.
///
/// Takes `&str` (shared read-only access) because parsing only needs to
/// inspect the line; the caller keeps ownership and the owned `String`
/// fields of `Sale` are created here from the borrowed slices.
///
/// Returns an `AppError` if any field is missing, extra fields are present,
/// or a field has an invalid type.
pub fn parse_line(line: &str) -> Result<Sale, AppError> {
    let mut split = line.split(',');

    let id_str = split
        .next()
        .ok_or_else(|| AppError::MissingField("id".to_string()))?;

    let name_str = split
        .next()
        .ok_or_else(|| AppError::MissingField("name".to_string()))?;

    let region_str = split
        .next()
        .ok_or_else(|| AppError::MissingField("region".to_string()))?;

    let amount_str = split
        .next()
        .ok_or_else(|| AppError::MissingField("amount".to_string()))?;

    let active_str = split
        .next()
        .ok_or_else(|| AppError::MissingField("active".to_string()))?;

    if split.next().is_some() {
        return Err(AppError::UnexpectedField(line.to_string()));
    }

    let id = id_str.trim().parse::<u32>()?;
    let name = name_str.trim().to_string();
    let region = region_str.trim().to_string();
    let amount = amount_str.trim().parse::<f64>()?;
    let active = active_str.trim().parse::<bool>()?;

    Ok(Sale::new(id, name, region, amount, active))
}

/// Opens a CSV file, validates its header, and returns a lazy iterator of
/// parsed records. Lines are read and parsed on demand — nothing is
/// materialized here, so the caller decides when (and whether) to collect.
///
/// `impl Iterator` is returned instead of `Box<dyn Iterator>`: the concrete
/// type is statically known, so no heap allocation or dynamic dispatch is
/// needed.
pub fn read_file(
    file_path: &str,
) -> Result<impl Iterator<Item = Result<Sale, AppError>>, AppError> {
    let file = File::open(file_path)?;
    let mut lines = BufReader::new(file).lines();

    let header = lines
        .next()
        .ok_or_else(|| AppError::InvalidHeader("<empty file>".to_string()))??;

    if header.trim() != EXPECTED_HEADER {
        return Err(AppError::InvalidHeader(header));
    }

    Ok(lines.map(|line_result| {
        let line = line_result?;
        parse_line(&line)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_success() {
        let raw_line = "42, Mouse, EU, 25.50, true";
        let result = parse_line(raw_line);

        assert!(result.is_ok());

        let sale = result.unwrap();

        assert_eq!(sale.id, 42);
        assert_eq!(sale.name, "Mouse");
        assert_eq!(sale.region, "EU");
        assert_eq!(sale.amount, 25.50);
        assert!(sale.active);
    }

    #[test]
    fn test_parse_line_missing_fields() {
        let raw_line = "42, Mouse, EU, true";
        let result = parse_line(raw_line);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::MissingField(_)));
    }

    #[test]
    fn test_parse_line_unexpected_extra_field() {
        let raw_line = "42, Mouse, EU, 25.50, true, extra";
        let result = parse_line(raw_line);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::UnexpectedField(_)));
    }

    #[test]
    fn test_parse_line_invalid_number() {
        let raw_line = "42, Mouse, EU, FREE, true";
        let result = parse_line(raw_line);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidAmount(_)));
    }

    #[test]
    fn test_parse_line_invalid_bool() {
        let raw_line = "42, Mouse, EU, 50.50, active";
        let result = parse_line(raw_line);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidActive(_)));
    }

    #[test]
    fn test_parse_line_invalid_id() {
        let raw_line = "AC, Mouse, EU, 50.50, active";
        let result = parse_line(raw_line);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidId(_)));
    }

    /// Helper: writes `content` to a unique temp file and returns its path.
    fn write_temp_file(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("parse_lib_test_{name}.csv"));
        std::fs::write(&path, content).expect("test setup: temp dir must be writable");
        path
    }

    #[test]
    fn test_read_file_success() {
        let path = write_temp_file(
            "read_ok",
            "id,name,region,amount,active\n1,Alice,EU,1200.50,true\n2,Bob,US,850.00,false\n",
        );

        let records: Vec<Sale> = read_file(path.to_str().expect("test path is valid UTF-8"))
            .expect("file exists with valid header")
            .collect::<Result<_, _>>()
            .expect("all rows are valid");

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Alice");
        assert!(!records[1].active);
    }

    #[test]
    fn test_read_file_not_found() {
        let result = read_file("/nonexistent/path/data.csv");

        assert!(matches!(result, Err(AppError::IoError(_))));
    }

    #[test]
    fn test_read_file_invalid_header() {
        let path = write_temp_file("bad_header", "foo,bar\n1,Alice,EU,1200.50,true\n");

        let result = read_file(path.to_str().expect("test path is valid UTF-8"));

        assert!(matches!(result, Err(AppError::InvalidHeader(_))));
    }

    #[test]
    fn test_read_file_corrupt_row_surfaces_as_error() {
        let path = write_temp_file(
            "corrupt_row",
            "id,name,region,amount,active\n1,Alice,EU,1200.50,true\n2,Bob,US,NOT_A_NUMBER,true\n",
        );

        let result: Result<Vec<Sale>, AppError> =
            read_file(path.to_str().expect("test path is valid UTF-8"))
                .expect("header is valid")
                .collect();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidAmount(_)));
    }
}
