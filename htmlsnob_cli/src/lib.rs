use glob::glob;
use htmlsnob::ast::Area;
use htmlsnob::config;
use htmlsnob::lint;
use htmlsnob::registry::Registry;
use std::path::Path;

pub struct SimpleArgs {
    pub paths: Vec<String>,
    pub autofix: bool,
    pub config: Option<String>,
    pub ignore: Vec<String>,
    pub registry: Registry,
}

struct Runner {
    result: String,
}

pub fn run(args: SimpleArgs) -> (i32, String) {
    let mut runner = Runner {
        result: String::new(),
    };
    let status_code = runner.run(args);
    (status_code, runner.result)
}

impl Runner {
    fn run(&mut self, args: SimpleArgs) -> i32 {
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
                            Err(e) => self.result.push_str(&format!("Glob error: {}\n", e)),
                        }
                    }
                }
                Err(e) => self
                    .result
                    .push_str(&format!("Invalid glob pattern '{}': {}\n", file_path, e)),
            }
        }

        if matched_file_paths.is_empty() {
            self.result.push_str(&format!(
                "No files found matching the patterns: {:?}\n",
                args.paths
            ));
            return 1;
        }

        // Filter out ignored files
        if !args.ignore.is_empty() {
            matched_file_paths = filter_ignored_files(matched_file_paths, &args.ignore);
        }

        if matched_file_paths.is_empty() {
            self.result.push_str("All matching files were ignored\n");
            return 1;
        }

        let mut success = true;

        let mut config;
        if let Some(config_path) = args.config {
            config = config::Config::from_file(&config_path, &args.registry);
        } else {
            let config_string = include_str!("../../default_config/default_config.toml");
            config = config::Config::from_toml(config_string, &args.registry);
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
                self.result.push_str(&format!("{}:\n", file_path.display()));
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
                    self.result
                        .push_str(&format!("{}: {}\n", line_number, line));
                    self.result.push_str(&format!(
                        "{}  {} {}\n",
                        line_number_indentation,
                        range_to_string(warning.areas[0].clone()),
                        truncated_message
                    ));
                }
                self.result.push('\n');
            }

            if args.autofix {
                let output = htmlsnob::format(&ast, &config);
                self.result.push_str(&output);

                // Overwrite the file with the fixed content
                if let Err(e) = std::fs::write(&file_path, output) {
                    self.result.push_str(&format!(
                        "Failed to write to file '{}': {}\n",
                        file_path.display(),
                        e
                    ));
                    success = false;
                }
            }
        }

        if success {
            self.result.push_str("Success: No issues found\n");
            0
        } else {
            1
        }
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
