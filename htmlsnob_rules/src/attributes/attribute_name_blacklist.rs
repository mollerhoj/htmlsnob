use std::collections::HashMap;

use htmlsnob::ast::OpenTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an elements specfied attribute does not have any of the specified values.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    pub tags: HashMap<String, Vec<String>>, // tag_name, blacklisted_attributes
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Attribute `{name}` is not allowed".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();

        for attribute in open_tag.attributes.iter() {
            if !attribute.name.is_left() {
                return vec![];
            }
            let string_area = attribute.name.left().unwrap();

            let Some(blacklist) = self.tags.get(&open_tag.name) else {
                return vec![];
            };

            if !blacklist.contains(&string_area.content) {
                return vec![];
            }

            let message = dynamic_format(
                &self.error_message,
                &[("name", string_area.content.clone())],
            );

            warnings.push(Warning::from_area(
                &self.name,
                &self.kind,
                string_area.area.clone(),
                &message,
                self.severity.clone(),
            ))
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "attribute_name_blacklist"
        [rules.tags]
        div = ["onclick"]
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"container\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div onclick="alert('Hello')"></div>
             -------
             attribute_name_blacklist: Attribute `onclick` is not allowed
        "#,
            CONFIG,
            &registry(),
        )
    }
}
