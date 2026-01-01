use htmlsnob::ast::{CloseTag, Node};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Enforces that a tag does not have any of the specified ancestor tags at any level.
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
    "Tag `{tag}` is not allowed within `{ancestor}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        // TODO: Move this to ParseState?
        let open_tag = match &parse_state.ast[close_tag.open_tag_index?] {
            Node::OpenTag(open_tag) => open_tag,
            _ => panic!(
                "Expected OpenTag at index {}",
                close_tag.open_tag_index.unwrap()
            ),
        };

        let open_tag_names = parse_state.open_tag_names();

        for (tag_name, blacklist) in &self.tags {
            if open_tag.name == *tag_name {
                for ancestor in &open_tag_names {
                    if blacklist.contains(ancestor) {
                        return Some(Warning::from_areas(
                            &self.name,
                            &self.kind,
                            &[open_tag.area.clone(), close_tag.area.clone()],
                            &dynamic_format(
                                &self.error_message,
                                &[
                                    ("tag", open_tag.name.clone()),
                                    ("ancestor", (*ancestor).clone()),
                                ],
                            ),
                            self.severity.clone(),
                        ));
                    }
                }
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
        kind = "ancestor_blacklist"
        [rules.tags]
        form = ["table"]
    "#;

    #[test]
    fn good_case() {
        test_case("<form></form><table><tr></tr></table>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <table><tr><form> </form></tr></table>
                       ------ -------
            "#,
            CONFIG,
            &registry(),
        )
    }
}
