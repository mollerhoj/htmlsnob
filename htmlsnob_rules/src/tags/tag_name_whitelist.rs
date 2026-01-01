use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that the document only use the specified tags.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    error_message: String,
    /// If true, allows tags that contain a dash (`-`) even if they are no
    #[serde(default = "yes")]
    allow_if_dashed: bool,
}

fn yes() -> bool {
    true
}

fn default_error_message() -> String {
    "Tag `{name}` is not allowed".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        let name = match open_tag {
            Some(tag) => tag.name.clone(),
            None => match close_tag {
                Some(tag) => tag.name.clone(),
                None => return None,
            },
        };

        if self.tags.contains(&name.to_lowercase()) {
            return None;
        }

        if self.allow_if_dashed && name.contains('-') {
            return None;
        }

        let message = dynamic_format(&self.error_message, &[("name", name.clone())]);

        let mut areas = Vec::new();
        if let Some(open_tag) = open_tag {
            areas.push(open_tag.area.clone());
        }

        if let Some(close_tag) = close_tag {
            areas.push(close_tag.area.clone());
        }

        Some(Warning::from_areas(
            &self.name,
            &self.kind,
            &areas,
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
        kind = "tag_name_whitelist"
        tags = ["p"]
    "#;

    #[test]
    fn good_case() {
        test_case("<p></p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <font><p></p></font>
            ------       -------
                         tag_name_whitelist: Tag `font` is not allowed
            "#,
            CONFIG,
            &registry(),
        )
    }
}
