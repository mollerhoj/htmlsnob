use htmlsnob::ast::{Attribute, Either};
use htmlsnob::dynamic_format::dynamic_format;
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
    "Attribute name `{name}` must match the regexp `{regexp}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute(&self, attribute: &mut Attribute) -> Option<Warning> {
        if let Either::Right(_) = &attribute.name {
            return None;
        }
        let attribute_name = attribute.name.left().unwrap();
        let re = &self.regexp;

        if re.is_match(&attribute_name.content) {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", attribute_name.content.clone()),
                ("regexp", re.to_string()),
            ],
        );

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            attribute_name.area.clone(),
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
        kind = "attribute_name_regexp"
        regexp = "^[a-z_]+$"
    "#;

    #[test]
    fn good_case() {
        test_case("<div valid_name='value'></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div InvalidName='value'></div>
             -----------
             attribute_name_regexp: Attribute name `InvalidName` must match the regexp `^[a-z_]+$`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
