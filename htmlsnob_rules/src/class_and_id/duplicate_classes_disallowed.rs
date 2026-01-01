use std::collections::HashSet;

use htmlsnob::ast::Attribute;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that no element has the same class name more than once.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// If true, the rule will automatically reorder the class names
    #[serde(default)]
    pub autofix: bool,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Class name `{name}` appears more than once".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute_vec(&self, attribute: &mut Attribute) -> Vec<Warning> {
        if attribute.name.is_right() {
            return vec![];
        }
        let attribute_name = attribute.name.left().unwrap();
        if attribute_name.content != "class" {
            return vec![];
        }

        let mut seen_classes = HashSet::new();

        let mut warnings = Vec::new();
        if let Some(ref mut attribute_value) = attribute.value {
            for part in attribute_value.parts.iter() {
                if part.is_right() {
                    continue;
                }
                let string_area = part.left().unwrap();

                if !seen_classes.contains(&string_area.content) {
                    seen_classes.insert(string_area.content.clone());
                } else {
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
                    ));
                }
            }
        }

        if self.autofix {
            if let Some(ref mut attribute_value) = attribute.value {
                let mut unique_parts = Vec::new();
                let mut seen_classes_fix = HashSet::new();

                for part in attribute_value.parts.iter() {
                    if part.is_right() {
                        unique_parts.push(part.clone());
                        continue;
                    }
                    let string_area = part.left().unwrap();

                    if !seen_classes_fix.contains(&string_area.content) {
                        seen_classes_fix.insert(string_area.content.clone());
                        unique_parts.push(part.clone());
                    }
                }

                attribute_value.parts = unique_parts;
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
        template_language = "handlebars"

        [[rules]]
        kind = "duplicate_classes_disallowed"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"a b c\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
        <div class="a b a c"></div>
                        -
                        duplicate_classes_disallowed: Class name `a` appears more than once
        "#,
            r#"
        <div class="a b c"></div>
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_template_expressions() {
        test_case_autofix(
            r#"
        <div class="a {{ 'a' }} b a c"></div>
                                  -
                                  duplicate_classes_disallowed: Class name `a` appears more than once
        "#,
            r#"
        <div class="a {{ 'a' }} b c"></div>
        "#,
            CONFIG,
            &registry(),
        )
    }
}
