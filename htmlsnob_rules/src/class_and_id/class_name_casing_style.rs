use htmlsnob::ast::{Attribute, Either};
use htmlsnob::case_converter::CaseStyle;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that class names have a specific casing style.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    error_message: String,
    /// If true, the rule will automatically reorder the class names
    #[serde(default)]
    pub autofix: bool,
    pub case_style: CaseStyle,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Class name `{class_name}` should be in {case_style}, change to `{expected_name}`.".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute_vec(&self, attribute: &mut Attribute) -> Vec<Warning> {
        if let Either::Right(_) = &attribute.name {
            return vec![];
        }
        let attribute_name = attribute.name.left().unwrap();
        if attribute_name.content != "class" {
            return vec![];
        }

        if attribute.value.is_none() {
            return vec![];
        }

        let mut warnings = Vec::new();
        for class in attribute.value.as_mut().unwrap().parts.iter_mut() {
            if class.is_right() {
                continue;
            }
            let class = class.left_mut().unwrap();

            let converted = self.case_style.convert(&class.content);
            if class.content != converted {
                let message = dynamic_format(
                    &self.error_message,
                    &[
                        ("class_name", class.content.clone()),
                        ("case_style", self.case_style.to_string()),
                        ("expected_name", converted.clone()),
                    ],
                );
                warnings.push(Warning::from_area(
                    &self.name,
                    &self.kind,
                    class.area.clone(),
                    &message,
                    self.severity.clone(),
                ));
                if self.autofix {
                    class.content = converted;
                }
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::{test_case, test_case_autofix};

    const CONFIG: &str = r#"
        [[rules]]
        kind = "class_name_casing_style"
        case_style = "kebab_case"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"p-6 bg-pink\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
        <div class="bg-pink BgPink"></div>
                            ------
                            class_name_casing_style: Class name `BgPink` should be in kebab-case, change to `bg-pink`.
        "#,
            r#"
        <div class="bg-pink bg-pink"></div>
        "#,
            CONFIG,
            &registry(),
        )
    }
}
