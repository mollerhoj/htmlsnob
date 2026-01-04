use std::fs::File;
use std::io::{self};
mod test_block_reader;
use htmlsnob::config;
use htmlsnob::registry::Registry;
use test_block_reader::TestBlockReader;

// TODO: Use test_utils::test_case instead of TestBlockReader

#[test]
fn test_no_parse_errors() {
    let file = File::open("tests/fixtures/no_parse_errors.html").expect("Failed to open file");
    let no_parse_errors_reader = TestBlockReader::new(io::BufReader::new(file));
    let mut config = config::Config::from_toml("", &Registry::new());

    for (name, input, expected, _ranges) in no_parse_errors_reader {
        let (ast, warnings) = htmlsnob::lint(&input, &mut config);
        let output = htmlsnob::format(&ast, &config);

        assert_eq!(
            no_whitespace(&output),
            no_whitespace(&expected),
            "\n   name: {}\n  input:  {}\n  output: {}\nexpected: {}\n",
            name,
            input.trim(),
            output.trim(),
            expected.trim()
        );
        assert_eq!(
            warnings.len(),
            0,
            "\n   name: {}\n  input:  {}\n  warnings: {:?}\n",
            name,
            input.trim(),
            warnings
        );
    }
}

#[test]
fn test_parse_errors() {
    let file = File::open("tests/fixtures/parse_errors.html").expect("Failed to open file");
    let parse_errors_reader = TestBlockReader::new(io::BufReader::new(file));
    let mut config = config::Config::from_toml("", &Registry::new());

    for (name, input, expected, expected_ranges) in parse_errors_reader {
        let (ast, warnings) = htmlsnob::lint(&input, &mut config);
        let output = htmlsnob::format(&ast, &config);

        let mut warning_ranges: Vec<_> =
            warnings.iter().flat_map(|warning| &warning.areas).collect();

        warning_ranges.sort_by_key(|range| range.start.column);

        assert_eq!(
            no_whitespace(&output),
            no_whitespace(&expected),
            "\n   name: {}\n  input:  {}\n  output: {}\nexpected: {}\n",
            name,
            input.trim(),
            input.trim(),
            expected.trim()
        );

        for i in 0..warning_ranges.len() {
            let (start, end) = expected_ranges[i];
            let expected_range_str = ranges_to_string(&expected_ranges);
            assert!(
                warning_ranges.len() > i,
                "Missing warning for ranges:\n{}\n{}",
                input,
                expected_range_str
            );
            let warning_ranges = warning_ranges
                .iter()
                .map(|range| (range.start.column, range.end.column))
                .collect::<Vec<(usize, usize)>>();
            let actual_range_str = ranges_to_string(&warning_ranges);

            let warning_range = warning_ranges[i];

            let w_start = warning_range.0;
            let w_end = warning_range.1;

            assert_eq!(
                w_start, start,
                "Expected warning start column mismatch:\n{}\n{}\n{}",
                input, expected_range_str, actual_range_str
            );
            assert_eq!(
                w_end, end,
                "Expected warning start column mismatch:\n{}\n{}\n{}",
                input, expected_range_str, actual_range_str
            );
        }
    }
}

#[test]
fn test_formatting() {
    let file = File::open("tests/fixtures/formatting.html").expect("Failed to open file");
    let formatting_reader = TestBlockReader::new(io::BufReader::new(file));
    let mut config =
        config::Config::from_toml("template_language = 'handlebars'", &Registry::new());

    for (name, input, expected, _ranges) in formatting_reader {
        let (ast, _warnings) = htmlsnob::lint(&input, &mut config);
        let output = htmlsnob::format(&ast, &config);

        assert_eq!(
            &output, &expected,
            "\n   name: {}\n  input:  {}\n  output: {}\nexpected: {}\n",
            name, input, output, expected
        );
    }
}

#[test]
fn test_template_languages() {
    for file_name in [
        "eex.html.eex",
        "handlebars.html.hbs",
        "jinja2.html.jinja2",
        "liquid.html.liquid",
        "mustache.html.mustache",
        "erb.html.erb",
        //"twig.html.twig", // Not supported yet
    ]
    .iter()
    {
        dbg!("------------------------------");
        dbg!(file_name);
        dbg!("------------------------------");
        let file = File::open(format!("tests/fixtures/template_languages/{}", file_name)).unwrap();
        let format_reader = TestBlockReader::new(io::BufReader::new(file));

        let registry = Registry::new();
        let mut config = config::Config::from_toml("", &registry);
        let template_language =
            htmlsnob::template_language::TemplateLanguage::from_filename(file_name);
        config.options.template_language = template_language;

        // TODO: Set template_language in config everywhere

        for (name, input, expected, _ranges) in format_reader {
            let (ast, _warnings) = htmlsnob::lint(&input, &mut config);
            let output = htmlsnob::format(&ast, &config);

            assert_eq!(
                &output, &expected,
                "\n   name: {}\n------input:------\n{}\n------output:------\n{}\n------expected:------\n{}\n, ast: {:?}",
                name, input, output, expected, ast
            );
        }
    }
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
