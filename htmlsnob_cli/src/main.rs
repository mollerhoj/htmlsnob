use clap::Parser;
use glob::glob;
use htmlsnob::ast::Area;
use htmlsnob::config;
use htmlsnob::lint;
use std::path::Path;
use std::process;

/// Check HTML files for issues
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input paths
    #[arg(default_value = "**/*.html")]
    paths: Vec<String>,

    /// Correct the issues inline when possible
    #[arg(short, long)]
    autofix: bool,

    /// Config file
    #[arg(short, long)]
    config: Option<String>,

    /// Files or patterns to ignore (can be specified multiple times)
    #[arg(short, long)]
    ignore: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let mut matched_file_paths = Vec::new();

    for file_path in &args.paths {
        match glob(file_path) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            if path.is_file() {
                                // disregard directories
                                matched_file_paths.push(path)
                            }
                        }
                        Err(e) => eprintln!("Glob error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Invalid glob pattern '{}': {}", file_path, e),
        }
    }

    if matched_file_paths.is_empty() {
        eprintln!("No files found matching the patterns: {:?}", args.paths);
        return;
    }

    // Filter out ignored files
    if !args.ignore.is_empty() {
        matched_file_paths = filter_ignored_files(matched_file_paths, &args.ignore);
    }

    if matched_file_paths.is_empty() {
        eprintln!("All matching files were ignored");
        return;
    }

    let mut success = true;

    let registry = htmlsnob_rules::registry();

    let mut config;
    if let Some(config_path) = args.config {
        config = config::Config::from_file(&config_path, &registry);
    } else {
        let config_string = include_str!("../../default_config/default_config.toml");
        config = config::Config::from_toml(config_string, &registry);
    }

    for file_path in matched_file_paths {
        let content = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read file: {}", file_path.display()));

        let template_language = htmlsnob::template_language::TemplateLanguage::from_filename(
            file_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        );

        config.options.template_language = template_language;

        let (ast, warnings) = lint(&content, &mut config);

        if !warnings.is_empty() {
            success = false;
            println!("{}:", file_path.display());
            for warning in warnings {
                // Print the line of the file with the warning
                let line_number = warning.areas[0].start.line;
                let line_number_indentation = " ".repeat(line_number.to_string().len());
                let line = content.lines().nth(line_number).unwrap_or("");
                let mut truncated_message = warning.message.clone();
                if truncated_message.len() > 80 {
                    truncated_message.truncate(80);
                    truncated_message.push_str("...");
                }
                println!("{}: {}", line_number, line);
                println!(
                    "{}  {} {}",
                    line_number_indentation,
                    range_to_string(warning.areas[0].clone()),
                    truncated_message
                );
            }
            println!();
        }

        if args.autofix {
            let output = htmlsnob::format(&ast, &config);
            println!("{}", output);

            // Overwrite the file with the fixed content
            if let Err(e) = std::fs::write(&file_path, output) {
                eprintln!("Failed to write to file '{}': {}", file_path.display(), e);
                success = false;
            }
        }
    }

    if success {
        println!("Success: No issues found");
        process::exit(0);
    } else {
        process::exit(1);
    }
}

/// Filter out files that match any of the ignore patterns
fn filter_ignored_files<P: AsRef<Path>>(files: Vec<P>, ignore_patterns: &[String]) -> Vec<P> {
    let mut result = Vec::new();

    'file_loop: for file in files {
        let path_str = match file.as_ref().to_str() {
            Some(s) => s,
            None => {
                // Keep files with non-UTF8 paths, since we can't match them against patterns
                result.push(file);
                continue;
            }
        };

        // Check if this file matches any ignore pattern
        for pattern in ignore_patterns {
            if let Ok(matcher) = glob::Pattern::new(pattern) {
                if matcher.matches(path_str) {
                    // This file matches an ignore pattern, skip it
                    continue 'file_loop;
                }
            }
        }

        // If we get here, the file didn't match any ignore pattern
        result.push(file);
    }

    result
}

fn range_to_string(range: Area) -> String {
    let dashes = "-".repeat(range.end.column - range.start.column);
    format!("{:>width$}", dashes, width = range.end.column)
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_config() {
        let config_string = "";
        let registry = htmlsnob_rules::registry();
        let config = htmlsnob::config::Config::from_toml(config_string, &registry);

        // Check if the config has the expected default values
        assert_eq!(config.options.indent_size, 2);
        assert_eq!(config.options.max_line_length, 80);
        assert!(config.rules.is_empty());
    }
}
