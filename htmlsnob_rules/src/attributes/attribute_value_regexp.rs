use std::collections::HashMap;

use htmlsnob::ast::{Either, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::rule_utils::deserialize_regex::DeserializableRegex;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The regular expression that attribute values must match
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default)]
    tags: HashMap<String, HashMap<String, DeserializableRegex>>,
}

fn default_error_message() -> String {
    "Attribute value `{value}` must match the regexp `{regexp}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();

        // Get the regexps for this tag, if any
        let Some(tag_regexps) = self.tags.get(&open_tag.name) else {
            return warnings;
        };

        // Iterate through all attributes
        for attribute in open_tag.attributes.iter_mut() {
            // Only process attributes with string names (not template expressions)
            let Either::Left(attribute_name) = &attribute.name else {
                continue;
            };

            // Check if there's a regexp for this attribute
            let Some(regexp) = tag_regexps.get(&attribute_name.content) else {
                continue;
            };

            // Check the attribute value if it exists
            if let Some(ref mut attribute_value) = attribute.value {
                let value = match &attribute_value.parts[..] {
                    [] => "", // If parts is empty, consider it a single empty string
                    [Either::Left(part)] => &part.content,
                    _ => continue,
                };

                if regexp.is_match(value) {
                    continue;
                }

                let message = dynamic_format(
                    &self.error_message,
                    &[("value", value.to_string()), ("regexp", regexp.to_string())],
                );

                warnings.push(Warning::from_area(
                    &self.name,
                    &self.kind,
                    attribute_value.area.clone(),
                    &message,
                    self.severity.clone(),
                ));
            }
        }

        warnings
    }
}
#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        template_language = "handlebars"

        [[rules]]
        kind = "attribute_value_regexp"
        [rules.tags]
        div.class = "^[a-zA-Z0-9]+$"
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"test\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div class="test!@#"></div>
                   ---------
                   attribute_value_regexp: Attribute value `test!@#` must match the regexp `^[a-zA-Z0-9]+$`
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn good_case_template_expression() {
        test_case(
            "<div class=\"test {{ 'allowed!@#' }}\"></div>",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_empty_value() {
        test_case(
            r#"
            <div class=""></div>
                       --
                       attribute_value_regexp: Attribute value `` must match the regexp `^[a-zA-Z0-9]+$`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
