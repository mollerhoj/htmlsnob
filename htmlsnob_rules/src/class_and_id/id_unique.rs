use htmlsnob::ast::{Attribute, AttributeValue, Either, OpenTag, StringArea};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashSet;

/// Enforces that all id values are unique within the document.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The error message to display when the rule fails.
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    /// Maps IDs that appear more than once to their first occurrence index
    #[serde(default)]
    seen_ids: HashSet<String>,
}

fn default_error_message() -> String {
    "id value `{value}` is used more than once".to_string()
}

fn get_id(attribute: &Attribute) -> Option<&AttributeValue> {
    let Either::Left(attr_name) = &attribute.name else {
        return None;
    };

    if attr_name.content.to_lowercase() != "id" {
        return None;
    }

    let attr_value = attribute.value.as_ref()?;

    if attr_value.parts.len() != 1 {
        return None;
    }

    if attr_value.parts[0].is_right() {
        return None;
    };

    Some(attr_value)
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        // Look through all attributes for id attributes
        for attribute in &open_tag.attributes {
            if let Some(id) = get_id(attribute) {
                let string_area = id.parts[0].left().unwrap();

                if !self.seen_ids.contains(&string_area.content) {
                    continue;
                };

                let message = dynamic_format(
                    &self.error_message,
                    &[("value", string_area.content.clone())],
                );

                return Some(Warning::from_area(
                    &self.name,
                    &self.kind,
                    id.area.clone(),
                    &message,
                    self.severity.clone(),
                ));
            }
        }

        None
    }

    fn track_open_tag(&mut self, open_tag: &OpenTag, _parse_state: &ParseState) {
        for attribute in &open_tag.attributes {
            if let Some(id) = get_id(attribute) {
                let string_area = id.parts[0].left().unwrap();
                self.seen_ids.insert(string_area.content.clone());
            }
        }
    }

    fn reset_state(&mut self) {
        self.seen_ids.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "id_unique"
    "#;

    #[test]
    fn good_case2() {
        test_case("<div><p id=\"x\"></p></div>", CONFIG, &registry())
    }

    #[test]
    fn good_case() {
        test_case("<p id='a'></p><p id='b'></p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <p id='a'></p><p id='a'></p>
                            ---
                            id_unique: id value `a` is used more than once
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_multiple_duplicates() {
        test_case(
            r#"
        <p id='a'></p><p id='a'></p><p id='a'></p>
                            ---           ---
                            id_unique: id value `a` is used more than once
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_different_duplicates() {
        test_case(
            r#"
        <p id='a'></p><p id='b'></p><p id='a'></p><p id='b'></p>
                                          ---           ---
                                          id_unique: id value `a` is used more than once
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn good_case_no_ids() {
        test_case("<p></p><div></div>", CONFIG, &registry())
    }

    #[test]
    fn good_case_single_id() {
        test_case("<p id='unique'></p>", CONFIG, &registry())
    }
}
