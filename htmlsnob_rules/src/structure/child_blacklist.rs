use std::collections::HashMap;

use htmlsnob::ast::Element;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Checks if an element is NOT a blacklisted child of its parent element.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    /// A map of parent tag names to a list of forbidden child tag names
    tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
}

impl Rule {
    pub fn check(&self, element: &mut Element, ancestor_tag_names: &[String]) -> Vec<Warning> {
        if ancestor_tag_names.is_empty() {
            return Vec::new();
        }
        let parent_name = ancestor_tag_names.last().unwrap();

        let mut warnings = Vec::new();

        if let Some(blacklist) = self.tags.get(parent_name) {
            if blacklist.contains(&element.name) {
                warnings.push(Warning::from_element(
                    element,
                    &format!(
                        "Invalid child element: {} is forbidden in {}, blacklisted elements are: {}",
                        element.name,
                        parent_name,
                        blacklist.join(", ")
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
        kind = "child_blacklist"
        [rules.tags]
        ul = ["div"]
    "#;

    #[test]
    fn good_case_blacklist() {
        test_case("<ul><li></li></ul>", CONFIG)
    }

    #[test]
    fn bad_case_blacklist() {
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
        BLOCK = ["div", "p"]
        [[rules]]
        kind = "child_blacklist"
        [rules.tags]
        INLINE = ["BLOCK"]
    "#;

    #[test]
    fn good_case_expansions() {
        test_case("<div><p><span><a></a></span></p></div>", CONFIG_EXPANSIONS)
    }

    #[test]
    fn bad_case_expansions() {
        test_case(
            r#"
            <a><div><span><p> </p></span></div></a>
               -----      --- ----       ------
        "#,
            CONFIG_EXPANSIONS,
        )
    }
}
