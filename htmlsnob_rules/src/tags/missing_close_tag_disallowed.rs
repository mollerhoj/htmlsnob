use htmlsnob::ast::{Area, CloseTag, Comment, Doctype, OpenTag, TemplateExpression};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that open tags have a corresponding close tag.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default)]
    except_tags: Vec<String>,
}

fn default_error_message() -> String {
    "Open tag `{name}` is missing close tag".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        let open_tag = open_tag?;

        if self.except_tags.contains(&open_tag.name) {
            return None;
        }

        if open_tag.self_closed {
            return None;
        }

        if close_tag.is_some() {
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
        template_language = "handlebars"

        [[rules]]
        kind = "missing_close_tag_disallowed"
        except_tags = ["br"]
    "#;

    #[test]
    fn good_case() {
        test_case("<p></p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <p><br>
            ---
            missing_close_tag_disallowed: Open tag `p` is missing close tag
            "#,
            CONFIG,
            &registry(),
        )
    }
}
