use htmlsnob::ast::{Attribute, Either, StringArea};
use htmlsnob::case_converter::CaseStyle;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that all attribute names match a specified casing style.
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
    "Attribute name `{name}` should be in {style}, change to `{converted_name}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute(&self, attribute: &mut Attribute) -> Option<Warning> {
        if let Either::Right(_) = attribute.name {
            return None;
        }
        let attribute_name = attribute.name.left().unwrap();
        let converted_name = self.style.convert(&attribute_name.content);

        if attribute_name.content == converted_name {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", attribute_name.content.clone()),
                ("style", self.style.to_string()),
                ("converted_name", converted_name.clone()),
            ],
        );

        let warning = Some(Warning::from_area(
            &self.name,
            &self.kind,
            attribute_name.area.clone(),
            &message,
            self.severity.clone(),
        ));

        if self.autofix {
            attribute.name = Either::Left(StringArea {
                content: converted_name,
                area: attribute_name.area.clone(), // TODO: Fix?
            });
        }

        warning
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;
    use htmlsnob::test_utils::tests::test_case_autofix;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "attribute_name_casing_style"
        style = "kebab_case"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<div data-test=\"value\"></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
            <div dataTest="value"></div>
                 --------
                 attribute_name_casing_style: Attribute name `dataTest` should be in kebab-case, change to `data-test`
            "#,
            r#"<div data-test="value"></div>"#,
            CONFIG,
            &registry(),
        )
    }
}
