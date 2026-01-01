use htmlsnob::ast::Attribute;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Attribute name missing, expected a name before the = sign".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute(&self, attribute: &mut Attribute) -> Option<Warning> {
        if let Some(attribute_name) = attribute.name.left() {
            if attribute_name.content.is_empty() {
                return Some(Warning::from_area(
                    &self.name,
                    &self.kind,
                    attribute.area.clone(),
                    &self.error_message,
                    self.severity.clone(),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "attribute_name_missing"
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"my-class\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div ="my-class"></div>
             -----------
             attribute_name_missing: Attribute name missing, expected a name before the = sign
        "#,
            CONFIG,
            &registry(),
        )
    }
}
