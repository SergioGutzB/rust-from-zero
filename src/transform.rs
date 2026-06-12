use crate::parser::Sale;

/// Generic, lazy summation over anything that yields values convertible to
/// `f64`. The bounds are minimal: `T: Into<f64>` is all the conversion needs,
/// and accepting `IntoIterator` (instead of `&[T]`) lets callers pass a lazy
/// adapter chain without materializing an intermediate `Vec`.
pub fn summarize<T, I>(values: I) -> f64
where
    T: Into<f64>,
    I: IntoIterator<Item = T>,
{
    values.into_iter().map(Into::into).sum()
}

/// Demonstrates shared read-only access (`&[T]`).
/// Borrowing is enough here: we only need to inspect each sale to sum the
/// active amounts, and the caller keeps ownership to reuse the data later.
/// The whole chain (filter → map → sum via `summarize`) stays lazy.
pub fn calculate_total_revenue(records: &[Sale]) -> f64 {
    summarize(
        records
            .iter()
            .filter(|sale| sale.active)
            .map(|sale| sale.amount),
    )
}

/// Demonstrates in-place mutation (`&mut [T]`).
/// Chosen because we want to sort the data directly in memory
/// without allocating a new vector or cloning elements.
pub fn sort_sales_by_amount_desc(records: &mut [Sale]) {
    records.sort_by(|a, b| {
        b.amount
            .partial_cmp(&a.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Demonstrates consumption by value (`Vec<T>`).
/// Chosen because the surviving records are moved into the result without
/// cloning; the discarded ones are dropped. A borrowed input would force a
/// `.clone()` per surviving record.
pub fn filter_active_by_region(records: Vec<Sale>, target_region: &str) -> Vec<Sale> {
    records
        .into_iter()
        .filter(|sale| sale.active)
        .filter(|sale| sale.region == target_region)
        .collect()
}

/// Demonstrates consumption by value (`Vec<T>`).
/// Takes ownership because each record is transformed and returned: mutating
/// owned values in the `map` closure avoids cloning every `Sale`.
pub fn uppercase_names(records: Vec<Sale>) -> Vec<Sale> {
    records
        .into_iter()
        .map(|mut sale| {
            sale.name = sale.name.to_uppercase();
            sale
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Sale;

    fn sample_sales() -> Vec<Sale> {
        vec![
            Sale::new(1, "Mouse".to_string(), "EU".to_string(), 25.50, true),
            Sale::new(2, "Keyboard".to_string(), "US".to_string(), 79.99, true),
            Sale::new(3, "Monitor".to_string(), "EU".to_string(), 500.0, false),
            Sale::new(4, "Laptop".to_string(), "EU".to_string(), 1499.99, true),
        ]
    }

    #[test]
    fn test_summarize_integers() {
        let values = vec![1, 2, 3, 4];
        assert_eq!(summarize(values), 10.0);
    }

    #[test]
    fn test_summarize_floats_lazy_chain() {
        let total = summarize([1.5f32, 2.5f32].into_iter().filter(|&x| x > 2.0));
        assert_eq!(total, 2.5);
    }

    #[test]
    fn test_summarize_empty() {
        let values: Vec<i32> = vec![];
        assert_eq!(summarize(values), 0.0);
    }

    #[test]
    fn test_calculate_total_revenue_skips_inactive() {
        let sales = sample_sales();
        let total = calculate_total_revenue(&sales);

        // 25.50 + 79.99 + 1499.99 (Monitor is inactive)
        assert!((total - 1605.48).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_total_revenue_empty() {
        let sales: Vec<Sale> = vec![];
        assert_eq!(calculate_total_revenue(&sales), 0.0);
    }

    #[test]
    fn test_filter_active_by_region_success() {
        let result = filter_active_by_region(sample_sales(), "EU");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Mouse");
        assert_eq!(result[1].name, "Laptop");
    }

    #[test]
    fn test_filter_active_by_region_empty_input() {
        let result = filter_active_by_region(vec![], "EU");
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_active_by_region_not_found() {
        let result = filter_active_by_region(sample_sales(), "West");
        assert!(result.is_empty());
    }

    #[test]
    fn test_uppercase_names() {
        let result = uppercase_names(sample_sales());

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].name, "MOUSE");
        assert_eq!(result[3].name, "LAPTOP");
    }

    #[test]
    fn test_sort_sales_by_amount_desc() {
        let mut records = sample_sales();

        sort_sales_by_amount_desc(&mut records);

        assert_eq!(records[0].amount, 1499.99);
        assert_eq!(records[1].amount, 500.0);
        assert_eq!(records[2].amount, 79.99);
        assert_eq!(records[3].amount, 25.50);
    }
}
