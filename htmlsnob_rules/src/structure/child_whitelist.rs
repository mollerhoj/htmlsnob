use htmlsnob::ast::Element;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Checks if an element is a valid child of its parent element.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    /// A map of parent tag names to a list of allowed child tag names
    tags: HashMap<String, Vec<String>>,
    #[serde(default = "default_error_message")]
    error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "{child} is not allowed in {parent}, must be one of: {whitelist}".to_string()
}

impl Rule {
    pub fn check(&self, element: &mut Element, ancestor_tag_names: &[String]) -> Vec<Warning> {
        if ancestor_tag_names.is_empty() {
            return Vec::new();
        }
        let parent_name = ancestor_tag_names.last().unwrap();

        let mut warnings = Vec::new();

        // for category_name in categories_for(parent_name) {
        if let Some(whitelist) = self.tags.get(parent_name) {
            if !whitelist.contains(&element.name) {
                warnings.push(Warning::from_element(
                    element,
                    &dynamic_format(
                        &self.error_message,
                        &[
                            ("child", element.name.clone()),
                            ("parent", parent_name.clone()),
                            ("whitelist", whitelist.join(", ")),
                        ],
                    ),
                    self.severity.clone(),
                ));
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "child_whitelist"
        [rules.tags]
        ul = ["li"]
    "#;

    #[test]
    fn good_case() {
        test_case("<ul><li></li></ul>", CONFIG)
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <ul><div> </div></ul>
            ----- ------
        "#,
            CONFIG,
        )
    }

    const CONFIG_EXPANSIONS: &str = r#"
        [expansions]
        INLINE = ["span", "a"]
        [[rules]]
        kind = "child_whitelist"
        [rules.tags]
        p = ["INLINE"]
    "#;

    #[test]
    fn good_case_expansions() {
        test_case("<p><span><a></a></span></p>", CONFIG_EXPANSIONS)
    }

    #[test]
    fn bad_case_expansions() {
        test_case(
            r#"
            <p><div><span></span></div></p>
               -----             ------
        "#,
            CONFIG_EXPANSIONS,
        )
    }
}
