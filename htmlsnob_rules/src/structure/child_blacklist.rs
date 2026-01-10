use std::collections::HashMap;

use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an element does not have any of the specified direct child elements
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    /// A map of parent tag names to a list of forbidden child tag names
    tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "`{child_tag}` is not allowed within `{parent_tag}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &htmlsnob::parser::ParseState,
    ) -> Option<Warning> {
        let open_tag_names = parse_state.open_tag_names();
        let parent_tag_name = open_tag_names.last()?;
        let blacklist = self.tags.get(parent_tag_name)?;

        let child_tag_name = match (&open_tag, &close_tag) {
            (Some(open_tag), _) => &open_tag.name,
            (_, Some(close_tag)) => &close_tag.name,
            _ => panic!("Expected either OpenTag or CloseTag"),
        };

        if !blacklist.contains(child_tag_name) {
            return None;
        }

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
            &self
                .error_message
                .replace("{child_tag}", child_tag_name)
                .replace("{parent_tag}", parent_tag_name),
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
        kind = "child_blacklist"
        [rules.tags]
        ul = ["div"]
    "#;

    #[test]
    fn good_case_blacklist() {
        test_case("<ul><li></li></ul>", CONFIG, &registry());
    }

    #[test]
    fn bad_case_blacklist() {
        test_case(
            r#"
        <ul><div> </div></ul>
            ----- ------
            child_blacklist: `div` is not allowed within `ul`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
