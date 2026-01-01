use std::collections::HashMap;

use htmlsnob::ast::OpenTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::rule_utils::deserialize_regex::DeserializableRegex;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an elements attribute are in a specified whitelist.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    pub globals: Vec<DeserializableRegex>,
    pub tags: HashMap<String, Vec<DeserializableRegex>>, // tag_name, whitelisted_attributes
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "`{name}` not allowed, must be a global attribute or: `{whitelist}`".to_string()
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
                continue;
            }
            let string_area = attribute.name.left().unwrap();
            if string_area.content.is_empty() {
                continue;
            }

            let Some(whitelist) = self.tags.get(&open_tag.name) else {
                continue;
            };

            // Accept name if in list of global attributes
            if self
                .globals
                .iter()
                .any(|re| re.is_match(&string_area.content))
            {
                continue;
            }

            // Accept name if in whitelist for this tag
            if whitelist.iter().any(|re| re.is_match(&string_area.content)) {
                continue;
            }

            let message = dynamic_format(
                &self.error_message,
                &[
                    ("name", string_area.content.clone()),
                    (
                        "whitelist",
                        whitelist
                            .iter()
                            .map(|re| re.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                    ),
                ],
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
        kind = "attribute_name_whitelist"
        globals = ["class", "style", "data"]
        [rules.tags]
        input = ["type"]
    "#;

    #[test]
    fn good_case2() {
        test_case(
            "<input class type data-mydata></input>",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <input class id></input>
                         --
                         attribute_name_whitelist: `id` not allowed, must be a global attribute or: `type`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
