use htmlsnob::ast::{Attribute, AttributeValue, Either, StringArea};
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
    pub style: BooleanAttributeStyle,
    /// The error message to display when the rule fails.
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    attributes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanAttributeStyle {
    NoValue,
    EmptyValue,
    SameValue,
}

fn boolean_attribute_style_description(style: &BooleanAttributeStyle, name: &str) -> String {
    let str = match style {
        BooleanAttributeStyle::NoValue => "no value `{name}`",
        BooleanAttributeStyle::EmptyValue => "an empty value `{name}=\"\"`",
        BooleanAttributeStyle::SameValue => {
            "the same value as the attribute name `{name}=\"{name}\"`"
        }
    };
    dynamic_format(str, &[("name", name.to_string())])
}

fn default_error_message() -> String {
    "Boolean Attribute must have {description}".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute(&self, attribute: &mut Attribute) -> Option<Warning> {
        if let Either::Right(_) = attribute.name {
            return None;
        }

        if !self
            .attributes
            .contains(&attribute.name.left().unwrap().content)
        {
            return None;
        }

        let attribute_name = attribute.name.left().unwrap();

        // Check if attribute already matches desired style
        match self.style {
            BooleanAttributeStyle::NoValue => {
                attribute.value.as_ref()?;
            }
            BooleanAttributeStyle::EmptyValue => {
                if let Some(value) = &attribute.value {
                    if value.parts.is_empty() {
                        return None;
                    }
                }
            }
            BooleanAttributeStyle::SameValue => {
                if let Some(value) = &attribute.value {
                    let string_areas: Vec<StringArea> = value.string_areas();
                    if string_areas.len() == 1 && string_areas[0].content == attribute_name.content
                    {
                        return None;
                    }
                }
            }
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", attribute_name.content.clone()),
                (
                    "description",
                    boolean_attribute_style_description(&self.style, &attribute_name.content),
                ),
            ],
        );

        if self.autofix {
            attribute.value = match self.style {
                BooleanAttributeStyle::NoValue => None,
                BooleanAttributeStyle::EmptyValue => Some(AttributeValue {
                    start_quote: Some('"'), // TODO: Detect existing quote style
                    end_quote: Some('"'),   // TODO: Detect existing quote style
                    parts: vec![],
                    area: attribute.area.clone(),
                }),
                BooleanAttributeStyle::SameValue => Some(AttributeValue {
                    start_quote: Some('"'), // TODO: Detect existing quote style
                    end_quote: Some('"'),   // TODO: Detect existing quote style
                    parts: vec![Either::Left(StringArea {
                        content: attribute_name.content.clone(),
                        area: attribute.area.clone(),
                    })],
                    area: attribute.area.clone(),
                }),
            };
        }

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            attribute.area.clone(),
            &message,
            self.severity.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;
    use htmlsnob::test_utils::tests::test_case_autofix;

    const CONFIG_NO_VALUE: &str = r#"
        [[rules]]
        kind = "boolean_attribute_style"
        style = "no_value"
        attributes = ["checked"]
        autofix = true
    "#;

    #[test]
    fn good_case_no_value() {
        test_case(r#"<input checked>"#, CONFIG_NO_VALUE, &registry());
    }

    #[test]
    fn bad_case_no_value() {
        test_case_autofix(
            r#"
            <input checked="checked">
                   -----------------
                   boolean_attribute_style: Boolean Attribute must have no value `checked`
            "#,
            r#"<input checked>"#,
            CONFIG_NO_VALUE,
            &registry(),
        )
    }

    const CONFIG_EMPTY_VALUE: &str = r#"
        [[rules]]
        kind = "boolean_attribute_style"
        style = "empty_value"
        attributes = ["checked"]
        autofix = true
    "#;

    #[test]
    fn good_case_empty_value() {
        test_case(r#"<input checked="">"#, CONFIG_EMPTY_VALUE, &registry());
    }

    #[test]
    fn bad_case_empty_value() {
        test_case_autofix(
            r#"
            <input checked>
                   -------
                   boolean_attribute_style: Boolean Attribute must have an empty value `checked=""`
            "#,
            r#"<input checked="">"#,
            CONFIG_EMPTY_VALUE,
            &registry(),
        )
    }

    const CONFIG_SAME_VALUE: &str = r#"
        [[rules]]
        kind = "boolean_attribute_style"
        style = "same_value"
        attributes = ["checked"]
        autofix = true
    "#;

    #[test]
    fn good_case_same_value() {
        test_case(
            r#"<input checked="checked">"#,
            CONFIG_SAME_VALUE,
            &registry(),
        );
    }

    #[test]
    fn bad_case_same_value() {
        test_case_autofix(
            r#"
            <input checked="">
                   ----------
                   boolean_attribute_style: Boolean Attribute must have the same value as the attribute name `checked="checked"`
            "#,
            r#"<input checked="checked">"#,
            CONFIG_SAME_VALUE,
            &registry(),
        )
    }
}
