use std::collections::HashMap;

use htmlsnob::ast::OpenTag;
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
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    // tag_name -> attribute_name -> whitelist
    tag_attributes: HashMap<String, HashMap<String, Vec<DeserializableRegex>>>, // (tag_name, attribute_name, whitelist)
    // attribute_name -> whitelist
    global_attributes: HashMap<String, Vec<DeserializableRegex>>, // (attribute_name, whitelist)
}

fn default_error_message() -> String {
    "Attribute value `{value}` must be one of `{whitelist}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();

        for attribute in &open_tag.attributes {
            let Some(attribute_name) = attribute.name.left() else {
                continue;
            };

            let mut whitelist = None;

            // Global attribute whitelist
            if let Some(global_attr_whitelist) = self.global_attributes.get(&attribute_name.content)
            {
                whitelist = Some(global_attr_whitelist);
            }

            // Tag-specific attribute whitelist
            if let Some(tag_attrs) = self.tag_attributes.get(&open_tag.name) {
                if let Some(tag_attr_whitelist) = tag_attrs.get(&attribute_name.content) {
                    whitelist = Some(tag_attr_whitelist);
                }
            }

            // If no whitelist found, skip validation
            let Some(whitelist) = whitelist else {
                continue;
            };

            if let Some(attribute_value) = &attribute.value {
                for part in &attribute_value.parts {
                    if part.is_right() {
                        continue;
                    }
                    let string_area = part.left().unwrap();

                    // If the attribute value is in the whitelist, continue
                    if whitelist.iter().any(|re| re.is_match(&string_area.content)) {
                        continue;
                    }

                    let message = dynamic_format(
                        &self.error_message,
                        &[
                            ("value", string_area.content.clone()),
                            (
                                "whitelist",
                                whitelist
                                    .iter()
                                    .map(|re| re.as_str())
                                    .collect::<Vec<_>>()
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
                    ));
                }
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
        kind = "attribute_value_whitelist"
        [rules.tag_attributes]
        a.target = ["_blank", "_self", "_parent", "_top"]
        button.command = ["toggle-popover", "show-popover", "hide-popover", "close", "request-close", "show-modal", "--.+"]
        [rules.global_attributes]
        autocorrect = ["on", "off"]
    "#;

    #[test]
    fn good_case() {
        test_case("<a target=\"_blank\"></a>", CONFIG, &registry())
    }

    #[test]
    fn good_case_regex() {
        test_case("<button command=\"--mycommand\"></a>", CONFIG, &registry())
    }

    #[test]
    fn good_case_only_a_tag() {
        test_case("<div target=\"invalid\"></div>", CONFIG, &registry())
    }

    #[test]
    fn good_case_global() {
        test_case("<a autocorrect=\"on\"></a>", CONFIG, &registry())
    }

    #[test]
    fn bad_case_global() {
        test_case(
            r#"
        <a autocorrect="invalid"></div>
                        -------
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <a target="invalid"></div>
                   ------- 
                   attribute_value_whitelist: Attribute value `invalid` must be one of `_blank, _self, _parent, _top`
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
    fn bad_case_template_expression() {
        test_case(
            r#"
        <a target="_blank {{ 'allowed' }} invalid"></div>
                                          -------
        "#,
            CONFIG,
            &registry(),
        )
    }
}
