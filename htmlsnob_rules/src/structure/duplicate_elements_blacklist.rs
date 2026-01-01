use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Checks if an element that only allowed once occurs more than once.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    /// A list of tag names that are not allowed to occur more than once
    tags: Vec<String>,
    #[serde(default)]
    suspect_indexes: HashMap<String, Vec<usize>>,
}

fn default_error_message() -> String {
    "`{tag}` is not allowed to occur more than once".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        open_tag?;

        if !self.tags.contains(&open_tag?.name) {
            return None;
        }

        if self.suspect_indexes[&open_tag?.name].len() <= 1 {
            return None;
        }

        let mut areas = vec![open_tag?.area.clone()];
        if let Some(close_tag) = close_tag {
            areas.push(close_tag.area.clone());
        }

        Some(Warning::from_areas(
            &self.name,
            &self.kind,
            &areas,
            &self.error_message.replace("{tag}", &open_tag?.name),
            self.severity.clone(),
        ))
    }

    fn track_open_tag(&mut self, open_tag: &OpenTag, _parse_state: &ParseState) {
        if self.tags.contains(&open_tag.name) {
            self.suspect_indexes
                .entry(open_tag.name.clone())
                .or_default()
                .push(open_tag.index);
        }
    }

    fn reset_state(&mut self) {
        self.suspect_indexes.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind  = "duplicate_elements_blacklist"
        tags = ["title", "main", "head", "body", "html", "footer", "header"]
    "#;

    #[test]
    fn good_case() {
        test_case("<body><title></title></body>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <title> </title> <title> <title> </title>
                         ------- ------- --------
                         duplicate_elements_blacklist: `title` is not allowed to occur more than once
        "#,
            CONFIG,
            &registry(),
        )
    }
}
