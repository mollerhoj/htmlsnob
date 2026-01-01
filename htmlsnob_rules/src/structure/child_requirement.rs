use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Rule that requires parent elements to have specific child elements.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// Map of parent tag names to required child elements
    pub tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    tag_missing_children: HashMap<usize, Vec<String>>, // (tag_index, missing_children)
}

fn default_error_message() -> String {
    "Must contain `{missing_children}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        let open_tag = open_tag?;

        if !self.tags.contains_key(&open_tag.name) {
            return None;
        }

        let mut missing_children: Vec<_> = self
            .tag_missing_children
            .get(&open_tag.index)
            .unwrap_or(&Vec::new())
            .clone();
        missing_children.sort();

        if missing_children.is_empty() {
            return None;
        }

        let mut areas = vec![open_tag.area.clone()];

        if let Some(close_tag) = close_tag {
            areas.push(close_tag.area.clone());
        }

        Some(Warning::from_areas(
            &self.name,
            &self.kind,
            &areas,
            &dynamic_format(
                &self.error_message,
                &[("missing_children", missing_children.join(", "))],
            ),
            self.severity.clone(),
        ))
    }

    fn track_open_tag(&mut self, open_tag: &OpenTag, parse_state: &ParseState) {
        let parent_index = parse_state.open_tag_indexes.last();
        if let Some(parent_index) = parent_index {
            if let Some(parent_tag) = self.tag_missing_children.get_mut(parent_index) {
                parent_tag.retain(|child_name| child_name != &open_tag.name);
            }
        }

        // Add to suspects if this tag requires children
        if self.tags.contains_key(&open_tag.name) {
            for required_child in &self.tags[&open_tag.name] {
                self.tag_missing_children
                    .entry(open_tag.index)
                    .or_default()
                    .push(required_child.clone());
            }
        }
    }

    fn reset_state(&mut self) {
        self.tag_missing_children.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "child_requirement"
        [rules.tags]
        html = ["head", "body"]
    "#;

    #[test]
    fn good_case() {
        test_case(
            "<html><head><title>Title</title></head><body></body></html>",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_must_be_immediate_child() {
        test_case(
            r#"
            <html><p><head></head></p><body></body></html>
            ------                                 -------
            child_requirement: Must contain `head`
            "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <html><body></body></html>
            ------             -------
            child_requirement: Must contain `head`
            "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_multiple_missing() {
        test_case(
            r#"
            <html><p></p></html>
            ------       -------
            child_requirement: Must contain `body, head`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
