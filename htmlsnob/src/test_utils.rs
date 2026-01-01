pub mod tests {
    use crate::{config::Config, format, lint, registry::Registry};

    pub fn test_case(case: &str, config_str: &str, registry: &Registry) {
        test_case_autofix(case, case, config_str, registry);
    }

    pub fn test_case_autofix(case: &str, expected: &str, config_str: &str, registry: &Registry) {
        let mut config = Config::from_toml(config_str, registry);

        let (input, expected_ranges, expected_error_message) = parse_case(case);
        let (expected, _, _) = parse_case(expected);
        let (ast, warnings) = lint(&input, &mut config);
        let output = format(&ast, &config);

        assert_eq!(
            no_whitespace(&output),
            no_whitespace(&expected),
            "`output` and `expected` are not equal\n  input:  {}\n  output: {}\nexpected: {}\nast: {:?}",
            input.trim(),
            output.trim(),
            expected.trim(),
            ast
        );

        for warning in &warnings {
            if warning.message.contains('{') || warning.message.contains('}') {
                panic!(
                    "Warning message contains {{ or }}: {}, is dynamic template invalid?",
                    warning.message
                );
            }
        }

        let warning_areas: Vec<_> = warnings.iter().flat_map(|warning| &warning.areas).collect();

        let mut warning_ranges = warning_areas
            .iter()
            .map(|area| (area.start.column, area.end.column))
            .collect::<Vec<_>>();
        warning_ranges.sort_by(|a, b| a.0.cmp(&b.0));

        let actual_range_str = ranges_to_string(&warning_ranges);
        let expected_range_str = ranges_to_string(&expected_ranges);

        assert_eq!(
            warning_areas.len(),
            expected_ranges.len(),
            "Expected warning count mismatch:\n{}\n{}\n{}",
            input,
            actual_range_str,
            expected_range_str,
        );

        assert_eq!(
            actual_range_str, expected_range_str,
            "Expected warning ranges mismatch:\n{}\n{}\n{}",
            input, actual_range_str, expected_range_str
        );

        if let Some(expected_error_message) = expected_error_message {
            let actual_error_message = format!("{}: {}", warnings[0].name, warnings[0].message);
            assert_eq!(
                actual_error_message, expected_error_message,
                "Expected warning message mismatch:\n{}\n{}\n{}",
                input, warnings[0].message, expected_error_message
            );
        }
    }

    fn parse_case(case: &str) -> (String, Vec<(usize, usize)>, Option<String>) {
        let mut ranges = Vec::new();
        let mut offset = 0;
        let lines: Vec<_> = case.lines().collect();
        let mut is_error_message_line = false;
        let mut error_message = None;

        let input = lines
            .iter()
            .filter_map(|line| {
                if is_error_message_line && !line.trim().is_empty() {
                    error_message = Some(line.trim().to_string());
                    is_error_message_line = false;
                    return None;
                }

                if line.chars().all(|c| c == ' ' || c == '-') && line.chars().any(|c| c == '-') {
                    ranges = extract_ranges(line, offset);
                    is_error_message_line = true;
                    None
                } else {
                    offset = line.chars().take_while(|&c| c == ' ').count();
                    Some(*line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        (input, ranges, error_message)
    }

    fn extract_ranges(line: &str, offset: usize) -> Vec<(usize, usize)> {
        // Find the ranges of consecutive '-' characters
        let mut ranges = Vec::new();
        let mut start_index = None;

        for (i, char) in line.chars().enumerate() {
            if char == '-' {
                if start_index.is_none() {
                    start_index = Some(i);
                }
            } else if let Some(start) = start_index {
                ranges.push((start - offset, i - offset));
                start_index = None;
            }
        }

        // Handle case where '-' is at the end of the line
        if let Some(start) = start_index {
            ranges.push((start - offset, line.len() - offset));
        }

        ranges
    }

    fn ranges_to_string(ranges: &Vec<(usize, usize)>) -> String {
        let mut range_str = String::new();
        let mut i = 0;
        for (start, end) in ranges {
            while i < *start {
                range_str.push(' ');
                i += 1;
            }
            while i < *end {
                range_str.push('-');
                i += 1;
            }
        }
        range_str
    }

    fn no_whitespace(input: &str) -> String {
        input
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }
}
