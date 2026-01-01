use regex::Regex;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStyle {
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    UpperCase,
}

impl CaseStyle {
    pub fn convert(&self, input: &str) -> String {
        match self {
            CaseStyle::CamelCase => to_camel_case(input),
            CaseStyle::PascalCase => to_pascal_case(input),
            CaseStyle::SnakeCase => to_snake_case(input),
            CaseStyle::KebabCase => to_kebab_case(input),
            CaseStyle::UpperCase => to_upper_case(input),
        }
    }
}

impl fmt::Display for CaseStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CaseStyle::CamelCase => write!(f, "camelCase"),
            CaseStyle::PascalCase => write!(f, "PascalCase"),
            CaseStyle::SnakeCase => write!(f, "snake_case"),
            CaseStyle::KebabCase => write!(f, "kebab-case"),
            CaseStyle::UpperCase => write!(f, "UPPER_CASE"),
        }
    }
}

// TODO: Optimize by returning Vec<String> directly instead of String
/// Transforms a string by adding spaces at various boundaries
fn transform_boundaries(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // First, replace hyphens with underscores
    let mut result = input.replace(['-', '_'], "|");

    // Insert underscores between lowercase and uppercase letters (camelCase and PascalCase)
    let re_camel = Regex::new(r"([a-z])([A-Z])").unwrap();
    result = re_camel.replace_all(&result, "$1|$2").to_string();

    // Insert underscores between uppercase sequences and uppercase followed by lowercase (XMLHttpRequest)
    let re_upper = Regex::new(r"([A-Z])([A-Z][a-z])").unwrap();
    result = re_upper.replace_all(&result, "$1|$2").to_string();

    result
}

/// Normalizes the string by removing extra spaces and converting to lowercase
fn normalize_words(input: &str) -> Vec<String> {
    transform_boundaries(input)
        .split('|')
        .map(|s| s.to_lowercase())
        .collect()
}

/// Converts a string to camelCase format
pub fn to_camel_case(input: &str) -> String {
    let words = normalize_words(input);
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].clone();
    for word in words.iter().skip(1) {
        let capitalized = match word.chars().next() {
            Some(c) => c.to_uppercase().to_string() + &word[c.len_utf8()..],
            None => String::new(),
        };
        result.push_str(&capitalized);
    }

    result
}

/// Converts a string to PascalCase format
pub fn to_pascal_case(input: &str) -> String {
    let words = normalize_words(input);

    let mut result = String::new();
    for word in words {
        let capitalized = match word.chars().next() {
            Some(c) => c.to_uppercase().to_string() + &word[c.len_utf8()..],
            None => String::new(),
        };
        result.push_str(&capitalized);
    }

    result
}

/// Converts a string to snake_case format
pub fn to_snake_case(input: &str) -> String {
    normalize_words(input).join("_")
}

/// Converts a string to kebab-case format
pub fn to_kebab_case(input: &str) -> String {
    normalize_words(input).join("-")
}

/// Converts a string to UPPER_CASE format
pub fn to_upper_case(input: &str) -> String {
    normalize_words(input).join("_").to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_kebab_case(""), "");
        assert_eq!(to_upper_case(""), "");
    }

    #[test]
    fn test_camel_case_conversion() {
        assert_eq!(to_camel_case("hello world"), "hello world");
        assert_eq!(to_camel_case("Hello World"), "hello world");
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(to_camel_case("HELLO_WORLD"), "helloWorld");
        assert_eq!(to_camel_case("XMLHttpRequest"), "xmlHttpRequest");
    }

    #[test]
    fn test_pascal_case_conversion() {
        //assert_eq!(to_pascal_case("hello world"), "Hello World"); TODO!
        //assert_eq!(to_pascal_case("Hello World"), "Hello World"); TODO!
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("HELLO_WORLD"), "HelloWorld");
        assert_eq!(to_pascal_case("XMLHttpRequest"), "XmlHttpRequest");
    }

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(to_snake_case("hello world"), "hello world");
        assert_eq!(to_snake_case("Hello World"), "hello world");
        assert_eq!(to_snake_case("hello_world"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("HELLO_WORLD"), "hello_world");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
    }

    #[test]
    fn test_kebab_case_conversion() {
        assert_eq!(to_kebab_case("hello world"), "hello world");
        assert_eq!(to_kebab_case("Hello World"), "hello world");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("hello-world"), "hello-world");
        assert_eq!(to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("HELLO_WORLD"), "hello-world");
        assert_eq!(to_kebab_case("XMLHttpRequest"), "xml-http-request");
    }

    #[test]
    fn test_upper_case_conversion() {
        assert_eq!(to_upper_case("hello world"), "HELLO WORLD");
        assert_eq!(to_upper_case("Hello World"), "HELLO WORLD");
        assert_eq!(to_upper_case("hello_world"), "HELLO_WORLD");
        assert_eq!(to_upper_case("hello-world"), "HELLO_WORLD");
        assert_eq!(to_upper_case("helloWorld"), "HELLO_WORLD");
        assert_eq!(to_upper_case("HelloWorld"), "HELLO_WORLD");
        assert_eq!(to_upper_case("HELLO_WORLD"), "HELLO_WORLD");
        assert_eq!(to_upper_case("XMLHttpRequest"), "XML_HTTP_REQUEST");
    }
}
