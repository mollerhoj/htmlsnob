use std::collections::HashMap;

use htmlsnob::ast::{CloseTag, Node, OpenTag};
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that the children of an element is in a particular order
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    /// A map of parent tag names to a list of forbidden child tag names
    tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "`{first_tag}` must come before `{second_tag}`".to_string()
}

// TODO: Consider: What is the tag is not closed?
// If either open or close tag is missing, we can ignore this.
// But: It means that the child_indexes approach may be flawed?
// 1) Once we realize a tag is not closed, we can more it's child indexes to the parent?
// 2) Or we can track in this rule somehow? More testing needed..
//
//
// ISSUE: Quite a few rules rely on knowing its parent tag. But it can only know once the parent
// tag has been closed...
// Offending rules:
// - text_disallowed
// - child_whitelist
// - child_blacklist
// - child_requirement
// - content_model_categories
// - ... and all other rules using the open_tag_index field..
//
//
//

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &htmlsnob::parser::ParseState,
    ) -> Option<Warning> {
        None
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::registry;
//     use htmlsnob::test_utils::tests::test_case;
//
//     const CONFIG: &str = r#"
//         [[rules]]
//         kind = "child_order"
//         [rules.tags]
//         html = ["head", "body"]
//     "#;
//
//     #[test]
//     fn good_case_blacklist() {
//         test_case(
//             "<html><head></head><body></body></html>",
//             CONFIG,
//             &registry(),
//         );
//     }
//
//     #[test]
//     fn bad_case_blacklist() {
//         test_case(
//             r#"
//             <html><body></body><head></head></html>
//                                ----- ------
//                                child_order: `head` must come before `body`
//             "#,
//             CONFIG,
//             &registry(),
//         )
//     }
// }
