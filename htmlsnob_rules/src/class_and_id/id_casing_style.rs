use htmlsnob::ast::{Attribute, Either, StringArea};
use htmlsnob::case_converter::CaseStyle;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that all id values match a specified casing style.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// If true, the rule will automatically convert to the prefered case style
    #[serde(default)]
    pub autofix: bool,
    /// The desired casing style for attribute names.
    pub style: CaseStyle,
    /// The error message to display when the rule fails.
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "id value `{value}` should be in {style}, change to `{converted_value}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute_vec(&self, attribute: &mut Attribute) -> Vec<Warning> {
        // Return early if not an "id" attribute
        if let Either::Right(_) = attribute.name {
            return vec![];
        }
        let attribute_name = attribute.name.left().unwrap();
        if attribute_name.content.to_lowercase() != "id" {
            return vec![];
        }

        let mut warnings = Vec::new();
        if let Some(ref mut attribute_value) = attribute.value {
            for part in attribute_value.parts.iter_mut() {
                if part.is_right() {
                    continue;
                }
                let string_area = part.left().unwrap();
                let converted_value = self.style.convert(&string_area.content);

                if string_area.content == converted_value {
                    continue;
                }

                let message = dynamic_format(
                    &self.error_message,
                    &[
                        ("value", string_area.content.clone()),
                        ("style", self.style.to_string()),
                        ("converted_value", converted_value.clone()),
                    ],
                );

                warnings.push(Warning::from_area(
                    &self.name,
                    &self.kind,
                    string_area.area.clone(),
                    &message,
                    self.severity.clone(),
                ));

                if self.autofix {
                    *part = Either::Left(StringArea {
                        content: converted_value,
                        area: string_area.area.clone(), // TODO: Fix broken area
                    });
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
    use htmlsnob::test_utils::tests::test_case_autofix;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "id_casing_style"
        style = "kebab_case"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<div id=\"value\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
        <div id="dataTest"></div>
                 --------
                 id_casing_style: id value `dataTest` should be in kebab-case, change to `data-test`
        "#,
            r#"<div id="data-test"></div>"#,
            CONFIG,
            &registry(),
        )
    }
}
