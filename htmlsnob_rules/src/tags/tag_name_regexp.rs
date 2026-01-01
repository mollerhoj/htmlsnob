use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::rule_utils::deserialize_regex::deserialize_regex;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use regex::Regex;
use serde::Deserialize;

/// Enforces that all tag names match a specified regular expression.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The regular expression that tag names must match
    #[serde(deserialize_with = "deserialize_regex")]
    pub regexp: Regex,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Tag name `{name}` must match the regexp `{regexp}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        let re = &self.regexp;

        if re.is_match(&open_tag.name) {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[("name", open_tag.name.clone()), ("regexp", re.to_string())],
        );

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            open_tag.area.clone(),
            &message,
            self.severity.clone(),
        ))
    }

    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        let re = &self.regexp;

        if re.is_match(&close_tag.name) {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[("name", close_tag.name.clone()), ("regexp", re.to_string())],
        );

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            close_tag.area.clone(),
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
        kind = "tag_name_regexp"
        regexp = "^[a-z]+$"
    "#;

    #[test]
    fn good_case() {
        test_case("<div><p>Hello</p></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <CustomElement>Hello</CustomElement>
            ---------------     ----------------
                                tag_name_regexp: Tag name `CustomElement` must match the regexp `^[a-z]+$`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
