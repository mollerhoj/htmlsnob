use htmlsnob::{
    ast::{Either, OpenTag},
    parser::ParseState,
    rule_trait::RuleTrait,
    Warning, WarningSeverity,
};
use serde::Deserialize;

/// Enforces that no attributes appear more than once in the same element.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Attribute \"{name}\" appears more than once".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();
        let mut seen_attributes = std::collections::HashSet::new();

        for attribute in open_tag.attributes.iter() {
            if let Either::Left(attribute_name) = &attribute.name {
                if seen_attributes.contains(&attribute_name.content) {
                    let message = self
                        .error_message
                        .replace("{name}", &attribute_name.content);

                    warnings.push(Warning::from_area(
                        &self.name,
                        &self.kind,
                        attribute_name.area.clone(),
                        &message,
                        self.severity.clone(),
                    ));
                } else {
                    seen_attributes.insert(attribute_name.content.clone());
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
        [[rules]]
        kind = "duplicate_attribute_names_disallowed"
    "#;

    #[test]
    fn good_case() {
        test_case("<div class=\"container\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <div class="container" class="main"></div>
                               -----
                               duplicate_attribute_names_disallowed: Attribute "class" appears more than once
        "#,
            CONFIG,
            &registry(),
        )
    }
}
