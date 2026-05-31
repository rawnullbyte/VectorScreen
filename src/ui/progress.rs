
/// Format a duration in seconds to a human-readable string.
///
/// Shows at most 2 time units, matching the Feather screen convention:
/// - `3661` → `"1h 1m"`
/// - `125` → `"2m 5s"`
/// - `65` → `"1m 5s"`
/// - `0` → `"0s"`
pub fn format_duration(seconds: u64) -> String {
    let units: &[(u64, &str)] = &[(86_400, "d"), (3_600, "h"), (60, "m"), (1, "s")];

    let mut remaining = seconds;
    let mut values = Vec::new();

    for &(divisor, unit) in units {
        if remaining >= divisor {
            let value = remaining / divisor;
            remaining %= divisor;
            values.push(format!("{}{}", value, unit));
        }
    }

    if values.is_empty() {
        return "0s".to_string();
    }

    // Show at most 2 most-significant units
    values.truncate(2);
    values.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_seconds() {
        assert_eq!(format_duration(0), "0s");
    }

    #[test]
    fn test_seconds_only() {
        assert_eq!(format_duration(5), "5s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_minutes_and_seconds() {
        assert_eq!(format_duration(65), "1m 5s");
        assert_eq!(format_duration(125), "2m 5s");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(3_599), "59m 59s");
    }

    #[test]
    fn test_hours_and_minutes() {
        assert_eq!(format_duration(3_661), "1h 1m");
        assert_eq!(format_duration(5_400), "1h 30m");
        assert_eq!(format_duration(7_200), "2h");
    }

    #[test]
    fn test_days() {
        assert_eq!(format_duration(86_400), "1d");
        assert_eq!(format_duration(90_000), "1d 1h");
    }

    #[test]
    fn test_large_values() {
        assert_eq!(format_duration(360_000), "4d 4h");
    }
}
