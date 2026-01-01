use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Enforces that the tag has one of the specified tags as an ancestor.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// A map of tag names to a list of disallowed ancestor tag names.
    tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
}

fn default_error_message() -> String {
    "`{tag}` must be a descendant of one of: `{required_ancestors}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        let open_tag = open_tag?;

        let open_tag_names = parse_state.open_tag_names();

        for (tag_name, required_ancestors) in &self.tags {
            if open_tag.name == *tag_name {
                if open_tag_names
                    .iter()
                    .any(|ancestor| required_ancestors.contains(ancestor))
                {
                    continue;
                }

                let mut areas = vec![open_tag.area.clone()];
                if let Some(close_tag) = close_tag {
                    areas.push(close_tag.area.clone());
                }

                return Some(Warning::from_areas(
                    &self.name,
                    &self.kind,
                    &areas,
                    &dynamic_format(
                        &self.error_message,
                        &[
                            ("tag", open_tag.name.clone()),
                            ("required_ancestors", required_ancestors.join(", ")),
                        ],
                    ),
                    self.severity.clone(),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "ancestor_requirement"
        [rules.tags]
        li = ["ul", "ol", "menu"]
    "#;

    #[test]
    fn good_case() {
        test_case("<ul><li></li></ul>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <li><ul></ul></li>
            ----         -----
            ancestor_requirement: `li` must be a descendant of one of: `ul, ol, menu`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
