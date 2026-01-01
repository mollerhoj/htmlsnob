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
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default)]
    order: Vec<DeserializableRegex>,
}

fn default_error_message() -> String {
    "Attribute `{first_name}` must be before `{second_name}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag_vec(
        &self,
        open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        let mut warnings = Vec::new();

        let mut last_index = 0;
        for attribute in open_tag.attributes.iter_mut() {
            let Either::Left(attribute_name) = &attribute.name else {
                continue;
            };

            if let Some(index) = self
                .order
                .iter()
                .position(|re| re.is_match(&attribute_name.content))
            {
                if index < last_index {
                    let message = dynamic_format(
                        &self.error_message,
                        &[
                            ("first_name", attribute_name.content.clone()),
                            ("second_name", self.order[last_index].to_string()),
                        ],
                    );
                    warnings.push(Warning::from_area(
                        &self.name,
                        &self.kind,
                        attribute_name.area.clone(),
                        &message,
                        self.severity.clone(),
                    ));
                }
                last_index = index;
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
        kind = "attributes_order"
        autofix = true
        order = ["class", "name", "data-.+", "src"]
    "#;

    #[test]
    fn good_case() {
        test_case(
            "<a class name ignore {{ 'class' }}></div>",
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <a name class ignore></div>
                -----
                attributes_order: Attribute `class` must be before `name`
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_regex() {
        test_case(
            r#"
        <a src data-test></div>
               ---------
               attributes_order: Attribute `data-test` must be before `src`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
