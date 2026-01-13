use htmlsnob::ast::OpenTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that only some tags are allowed to be self closing.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    pub except_tags: Vec<String>,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "`{name}` is not allowed to be self-closing".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        if !open_tag.self_closed {
            return None;
        }

        if self.except_tags.contains(&open_tag.name) {
            return None;
        }

        let message = dynamic_format(&self.error_message, &[("name", open_tag.name.clone())]);

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            open_tag.area.clone(),
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
        kind = "self_closing_tag_disallowed"
        except_tags = ["br"]
    "#;

    #[test]
    fn good_case() {
        test_case("<br/>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <p/>
            ----
            self_closing_tag_disallowed: `p` is not allowed to be self-closing
        "#,
            CONFIG,
            &registry(),
        )
    }
}
