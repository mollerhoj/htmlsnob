use htmlsnob::ast::{Position, Area, Text};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that certain tags do not contain content
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    severity: WarningSeverity,
    /// The list of tags that must not contain content
    tags: Vec<String>,
    #[serde(default = "default_error_message")]
    pub error_message: String,
}

fn default_error_message() -> String {
    "`{tag}` tags must not contain text content".to_string()
}

impl RuleTrait for Rule {
    fn apply_text(&self, text: &mut Text, parse_state: &ParseState) -> Option<Warning> {
        let parent_names = parse_state.open_tag_names();
        let parent_name = parent_names.last()?;

        if self.tags.contains(parent_name) {
            let message = dynamic_format(&self.error_message, &[("tag", parent_name.clone())]);

            
            return Some(Warning::from_areas(
                &self.name,
                &self.kind,
                &non_whitespace_areas(text),
                &message,
                self.severity.clone(),
            ));
        }

        None
    }
}

fn non_whitespace_areas(text: &Text) -> Vec<Area> {
    let mut start_column = text.area.start.column;
    let mut areas = Vec::new();
    for (index, line_text) in text.content.lines().enumerate() {
        let line = text.area.start.line + index;
        let end_column = start_column + line_text.len();

        let leading_whitespace = line_text.chars().take_while(|c| c.is_whitespace()).count();
        let trailing_whitespace = line_text.chars().rev().take_while(|c| c.is_whitespace()).count();

        areas.push(Area {
            start: Position {
                line: line,
                column: start_column + leading_whitespace
            },
            end: Position {
                line: line,
                column: end_column - trailing_whitespace
            },
        });

        start_column = 0;
    }
    areas
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "text_disallowed"
        tags = ["audio"]
        "#;

    #[test]
    fn good_case() {
        test_case("<audio></audio>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div><audio>Hello</audio></div>
                    ----- 
                    text_disallowed: `audio` tags must not contain text content
        "#,
            CONFIG,
            &registry(),
        )
    }
}
