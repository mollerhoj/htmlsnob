use htmlsnob::ast::Text;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::rule_utils::deserialize_regex::deserialize_regex;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use regex::Regex;
use serde::Deserialize;

/// Enforces that all attribute names match a specified regular expression.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The regular expression that attribute names must match
    #[serde(deserialize_with = "deserialize_regex")]
    pub regexp: Regex,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Text `{text}` must match the regexp `{regexp}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_text(&self, text: &mut Text, _parse_state: &ParseState) -> Option<Warning> {
        if self.regexp.is_match(&text.content) {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("text", text.content.clone()),
                ("regexp", self.regexp.to_string()),
            ],
        );

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            text.area.clone(),
            &message,
            self.severity.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "text_regexp"
        regexp = "^[a-z_]+$"
    "#;

    #[test]
    fn good_case() {
        test_case("<p>valid_name</div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <p>InvalidName</p>
           -----------
           text_regexp: Text `InvalidName` must match the regexp `^[a-z_]+$`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
