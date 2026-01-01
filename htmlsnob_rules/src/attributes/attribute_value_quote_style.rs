use htmlsnob::ast::{Attribute, Either};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that all attribute values use the same quote style.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Single,
    Double,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    // If true, the rule will automatically convert to the prefered quote style
    pub autofix: bool,
    /// style: "single" | "double" - The prefered quote style
    pub style: Style,
    /// error_message: The error message to display when the rule fails
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Attribute value must be quoted with {prefered_quote}{prefered_quote}".to_string()
}

impl RuleTrait for Rule {
    fn apply_attribute(&self, attribute: &mut Attribute) -> Option<Warning> {
        attribute.value.as_ref()?;
        let value = attribute.value.as_mut().unwrap();

        let prefered_quote = match self.style {
            Style::Single => '\'',
            Style::Double => '\"',
        };

        if value.start_quote != Some(prefered_quote) || value.end_quote != Some(prefered_quote) {
            let message = dynamic_format(
                &self.error_message,
                &[("prefered_quote", prefered_quote.to_string())],
            );

            if self.autofix {
                value.start_quote = Some(prefered_quote);
                value.end_quote = Some(prefered_quote);
            }

            return Some(Warning::from_area(
                &self.name,
                &self.kind,
                value.area.clone(),
                &message,
                self.severity.clone(),
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::{test_case, test_case_autofix};

    const CONFIG: &str = r#"
        [[rules]]
        kind = "attribute_value_quote_style"
        autofix = true
        style = "single"
    "#;

    #[test]
    fn good_case() {
        test_case("<div class='container'></div>", CONFIG, &registry())
    }

    #[test]
    fn missing_quotes() {
        test_case_autofix(
            r#"
        <div class=container></div>
                   ---------
                   attribute_value_quote_style: Attribute value must be quoted with ''
        "#,
            r#"
        <div class='container'></div>
        "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn quotes_double() {
        test_case_autofix(
            r#"
        <div class="container"></div>
                   -----------
        "#,
            r#"
        <div class='container'></div>
        "#,
            CONFIG,
            &registry(),
        )
    }
}
