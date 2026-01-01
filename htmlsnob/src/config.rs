use crate::registry::Registry;
use crate::rule_trait::RuleTrait;
use crate::serde_expander::expand_serde;
use crate::template_language::TemplateLanguage;
use serde::Deserialize;

#[derive(Debug)]
pub struct Config {
    pub rules: Vec<Box<dyn RuleTrait>>, // Placeholder for rule names or identifiers
    pub options: Options,
}

#[derive(Deserialize, Debug)]
pub struct Options {
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,
    #[serde(default)]
    pub template_language: TemplateLanguage,
}

fn default_indent_size() -> usize {
    2
}

fn default_max_line_length() -> usize {
    80
}

// TODO: Change panic to Result
impl Config {
    pub fn from_file(file_path: &str, registry: &Registry) -> Self {
        match std::fs::read_to_string(file_path) {
            Ok(content) => Self::from_toml(content.as_str(), registry),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    panic!("Configuration file not found: '{}' - Please check the file path and try again", file_path);
                } else {
                    panic!("Failed to load rules from '{}': {}", file_path, e);
                }
            }
        }
    }

    pub fn from_toml(toml_str: &str, registry: &Registry) -> Self {
        let mut value: toml::Value = toml::from_str(toml_str).expect("Failed to parse input TOML");
        expand_serde(&mut value);

        let rules_array = value
            .as_table()
            .and_then(|table| table.get("rules"))
            .and_then(|rules_value| rules_value.as_array());

        let mut rules = Vec::new();
        if let Some(rules_array) = rules_array {
            for rule in rules_array {
                let result = registry.build_rule_instance(rule.clone());

                if let Ok(rule_instance) = result {
                    rules.push(rule_instance);
                } else {
                    panic!(
                        "Failed to build rule instance from: {:?} {:?}",
                        rule, result
                    );
                }
            }
        }

        let options = Options::deserialize(value).expect("Failed to deserialize options from TOML");

        Config { options, rules }
    }
}
