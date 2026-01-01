use std::collections::HashMap;

use htmlsnob::ast::OpenTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an element has all of the specified attributes.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// lists of required attributes for different elements
    pub attributes: HashMap<String, Vec<String>>,
    /// error_message: The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Missing required attributes: {attributes}".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();

        if let Some(required_attributes) = self.attributes.get(&open_tag.name) {
            let element_attributes = open_tag
                .attributes
                .iter()
                .filter_map(|attr| attr.name.left())
                .map(|name| name.content.clone())
                .collect::<Vec<_>>();

            // Check if any required attributes are missing
            let missing_attributes: Vec<_> = required_attributes
                .iter()
                .filter(|attr| !element_attributes.contains(attr))
                .cloned()
                .collect();

            if !missing_attributes.is_empty() {
                let message = dynamic_format(
                    &self.error_message,
                    &[("attributes", missing_attributes.join(", "))],
                );
                warnings.push(Warning::from_area(
                    &self.name,
                    &self.kind,
                    open_tag.area.clone(),
                    &message,
                    self.severity.clone(),
                ));
            }
        }

        warnings
    }

    //pub fn check(&self, element: &mut Element) -> Vec<Warning> {
    //    let mut warnings = Vec::new();

    //    if let Some(opening_tag_area) = &element.opening_tag_area {
    //        if let Some(required_attributes) = self.attributes.get(&element.name) {
    //            let element_attributes = element
    //                .attributes
    //                .iter()
    //                .filter_map(|attr| attr.name.left())
    //                .map(|name| name.content.clone())
    //                .collect::<Vec<_>>();

    //            // Check if any required attributes are missing
    //            let missing_attributes: Vec<_> = required_attributes
    //                .iter()
    //                .filter(|attr| !element_attributes.contains(attr))
    //                .cloned()
    //                .collect();

    //            if !missing_attributes.is_empty() {
    //                let message = dynamic_format(
    //                    &self.error_message,
    //                    &[("attributes", missing_attributes.join(", "))],
    //                );
    //                warnings.push(Warning::from_area(
    //                    opening_tag_area.clone(),
    //                    &message,
    //                    self.severity.clone(),
    //                ));
    //            }
    //        }
    //    }

    //    warnings
    //}
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "attribute_name_requirement"
        [rules.attributes]
        img = ["src", "alt"]
    "#;

    #[test]
    fn good_case() {
        test_case(
            "<img src=\"image.png\" alt=\"Image\" />",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
                <img src="image.png" />
                -----------------------
                attribute_name_requirement: Missing required attributes: alt

        "#,
            CONFIG,
            &registry(),
        )
    }
}
