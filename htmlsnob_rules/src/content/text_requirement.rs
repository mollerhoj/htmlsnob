use htmlsnob::ast::{CloseTag, Node, OpenTag, Text};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Requires that some elements contain content
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    severity: WarningSeverity,
    /// The list of tags that must contain content
    tags: Vec<String>,
    #[serde(default)]
    suspect_indexes: Vec<usize>,
    #[serde(default = "default_error_message")]
    pub error_message: String,
}

fn default_error_message() -> String {
    "`{tag}` tags must contain text".to_string()
}

impl RuleTrait for Rule {
    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        if !self.tags.contains(&close_tag.name) {
            return None;
        }
        close_tag.open_tag_index?;

        let open_tag_index = close_tag.open_tag_index.unwrap();
        if !self.suspect_indexes.contains(&open_tag_index) {
            return None;
        }

        let Node::OpenTag(open_tag) = &parse_state.ast[open_tag_index] else {
            panic!("Expected OpenTag at index {}", open_tag_index);
        };

        let message = dynamic_format(&self.error_message, &[("tag", close_tag.name.clone())]);

        Some(Warning::from_areas(
            &self.name,
            &self.kind,
            &[open_tag.area.clone(), close_tag.area.clone()],
            &message,
            self.severity.clone(),
        ))
    }

    fn track_open_tag(&mut self, open_tag: &OpenTag, _parse_state: &ParseState) {
        if self.tags.contains(&open_tag.name) {
            self.suspect_indexes.push(open_tag.index);
        }
    }

    fn track_text(&mut self, _text: &Text, _parse_state: &ParseState) {
        self.suspect_indexes.clear();
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
        kind = "text_requirement"
        tags = ["p"]
        "#;

    #[test]
    fn good_case() {
        test_case("<p>Hello</p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div><p> </p></div>
             --- ----
            text_requirement: `p` tags must contain text
        "#,
            CONFIG,
            &registry(),
        )
    }
}
