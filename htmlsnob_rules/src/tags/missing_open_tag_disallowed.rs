use htmlsnob::ast::CloseTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that close tags have a corresponding open tag.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Close tag `{name}` is missing open tag".to_string()
}

impl RuleTrait for Rule {
    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        if close_tag.open_tag_index.is_some() {
            return None;
        }

        let message = dynamic_format(&self.error_message, &[("name", close_tag.name.clone())]);

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
        template_language = "handlebars"

        [[rules]]
        kind = "missing_open_tag_disallowed"
    "#;

    #[test]
    fn good_case() {
        test_case("<p></p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            </p>
            ----
            missing_open_tag_disallowed: Close tag `p` is missing open tag
            "#,
            CONFIG,
            &registry(),
        )
    }
}
