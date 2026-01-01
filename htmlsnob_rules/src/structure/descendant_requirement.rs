use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Rule that requires parent elements to have specific descendant elements.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// Map of parent tag names to required descendant elements
    pub tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    pub missing_descendants: HashMap<String, Vec<usize>>, // (missing_descendant_name, indexes)
}

fn default_error_message() -> String {
    "Must be a parent of `{missing_descendants}`".to_string()
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

        let mut missing_descendants: Vec<_> = self
            .missing_descendants
            .iter()
            .filter_map(|(descendant_name, indexes)| {
                if indexes.contains(&open_tag.index) {
                    Some(descendant_name.clone())
                } else {
                    None
                }
            })
            .collect();
        missing_descendants.sort();

        if missing_descendants.is_empty() {
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
                &[("missing_descendants", missing_descendants.join(", "))],
            ),
            self.severity.clone(),
        ))
    }

    fn track_open_tag(&mut self, open_tag: &OpenTag, _parse_state: &ParseState) {
        // Remove any suspects that have this tag as a required descendant
        self.missing_descendants.remove(&open_tag.name);

        // Add to suspects if this tag requires descendants
        if self.tags.contains_key(&open_tag.name) {
            for required_descendant in &self.tags[&open_tag.name] {
                self.missing_descendants
                    .entry(required_descendant.clone())
                    .or_default()
                    .push(open_tag.index);
            }
        }
    }

    fn reset_state(&mut self) {
        self.missing_descendants.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "descendant_requirement"
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
    fn bad_case() {
        test_case(
            r#"
            <html><body></body></html>
            ------             -------
            descendant_requirement: Must be a parent of `head`
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
            descendant_requirement: Must be a parent of `body, head`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
