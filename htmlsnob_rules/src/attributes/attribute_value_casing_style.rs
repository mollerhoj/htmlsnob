use htmlsnob::ast::{Attribute, Either, StringArea};
use htmlsnob::case_converter::CaseStyle;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The prefered case style for attribute values
    pub style: CaseStyle,
    /// If true, the rule will automatically convert to the prefered case style
    pub autofix: bool,
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Attribute value `{value}` should be in {style}, change to `{converted_value}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute_vec(&self, attribute: &mut Attribute) -> Vec<Warning> {
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
                        area: string_area.area.clone(),
                    });
                }
            }
        }
        warnings
    }
}

#[cfg(test)]
mod tests {
    use htmlsnob::test_utils::tests::{test_case, test_case_autofix};

    use crate::registry;

    const CONFIG: &str = r#"
        template_language = "handlebars"

        [[rules]]
        kind = "attribute_value_casing_style"
        autofix = true
        style = "kebab_case"
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"hello-world\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
        <div class="helloWorld"></div>
                    ----------  
                    attribute_value_casing_style: Attribute value `helloWorld` should be in kebab-case, change to `hello-world`
        "#,
            r#"<div class="hello-world"></div>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn good_case_template_expressions() {
        test_case(
            "<div class=\"hello-world {{ thisIsAllowed }}\"></div>",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_template_expressions() {
        test_case_autofix(
            r#"
        <div class="{{ isAllowed }} helloWorld {{ alsoAllowed }}"></div>
                                    ----------
                                    attribute_value_casing_style: Attribute value `helloWorld` should be in kebab-case, change to `hello-world`
        "#,
            r#"<div class="{{ isAllowed }} hello-world {{ alsoAllowed }}"></div>"#,
            CONFIG,
            &registry(),
        )
    }
}
