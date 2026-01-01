use htmlsnob::ast::{Attribute, Either};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

// TODO Support leaving some classes unordered
// TODO See: https://github.com/heybourn/headwind for ideas

/// Enforces that all class names are in a specified order.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    error_message: String,
    /// If true, the rule will automatically reorder the class names
    pub autofix: bool,
    /// The prefered order of class names
    pub order: Vec<String>,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "`{first_name}` must be before `{second_name}`".to_string()
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

        let mut last_index = 0;
        let mut warnings = Vec::new();
        for class in attribute.value.as_mut().unwrap().string_areas().iter() {
            if let Some(index) = self.order.iter().position(|name| name == &class.content) {
                if index < last_index {
                    let message = dynamic_format(
                        &self.error_message,
                        &[
                            ("first_name", class.content.clone()),
                            ("second_name", self.order[last_index].clone()),
                        ],
                    );
                    warnings.push(Warning::from_area(
                        &self.name,
                        &self.kind,
                        class.area.clone(),
                        &message,
                        self.severity.clone(),
                    ));
                }
                last_index = index;
            }
        }

        if self.autofix {
            let mut sorted_string_areas = attribute.value.as_ref().unwrap().string_areas().clone();
            sorted_string_areas.retain(|string_area| self.order.contains(&string_area.content));

            sorted_string_areas.sort_by(|a, b| {
                let index_a = self
                    .order
                    .iter()
                    .position(|name| name == &a.content)
                    .unwrap();
                let index_b = self
                    .order
                    .iter()
                    .position(|name| name == &b.content)
                    .unwrap();
                index_a.cmp(&index_b)
            });

            if let Some(value) = attribute.value.as_mut() {
                let mut iter = sorted_string_areas.into_iter();
                value.parts = value
                    .parts
                    .iter()
                    .map(|part| match part {
                        Either::Left(string_area) => {
                            if self.order.contains(&string_area.content) {
                                // If the class is in order, replace it with the sorted one
                                Either::Left(iter.next().unwrap().clone())
                            } else {
                                // If not in order, keep the original
                                Either::Left(string_area.clone())
                            }
                        }
                        Either::Right(template_expression) => {
                            Either::Right(template_expression.clone())
                        }
                    })
                    .collect();
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
        kind = "class_order"
        order = ["p-6", "bg-pink"]
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
        <div class="bg-pink my-class {{template_expression}} p-6"></div>
                                                             ---
                                                             class_order: `p-6` must be before `bg-pink`
        "#,
            r#"
        <div class="p-6 my-class {{template_expression}} bg-pink"></div>
        "#,
            CONFIG,
            &registry(),
        )
    }
}
