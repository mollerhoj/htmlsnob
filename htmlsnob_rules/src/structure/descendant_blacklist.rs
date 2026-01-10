use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
//use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::HashMap;

/// Enforces that some descendant are not allowed
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    pub tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub reversed_tags: HashMap<String, Vec<String>>,
}

fn default_error_message() -> String {
    "`{tag}` not allowed within `{ancestor_tag}`".to_string()
}

impl RuleTrait for Rule {
    fn build(&mut self) {
        for (ancestor_tag, descendant_tags) in &self.tags {
            for descendant_tag in descendant_tags {
                self.reversed_tags
                    .entry(descendant_tag.clone())
                    .or_default()
                    .push(ancestor_tag.clone());
            }
        }
    }

    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        let open_tag_names = parse_state.open_tag_names();

        let tag_name = match (&open_tag, &close_tag) {
            (Some(open_tag), _) => &open_tag.name,
            (_, Some(close_tag)) => &close_tag.name,
            _ => panic!("Expected either OpenTag or CloseTag"),
        };

        let blacklist = self.reversed_tags.get(tag_name)?;

        for ancestor in &open_tag_names {
            if blacklist.contains(ancestor) {
                let message = dynamic_format(
                    &self.error_message,
                    &[
                        ("tag", tag_name.clone()),
                        ("ancestor_tag", ancestor.clone()),
                    ],
                );

                let mut areas = Vec::new();
                if let Some(open_tag) = open_tag {
                    areas.push(open_tag.area.clone());
                }
                if let Some(close_tag) = close_tag {
                    areas.push(close_tag.area.clone());
                }

                return Some(Warning::from_areas(
                    &self.name,
                    &self.kind,
                    &areas,
                    &message,
                    self.severity.clone(),
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "descendant_blacklist"
        [rules.tags]
        a = ["a", ]
        address = ["address", "article", "aside", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "nav", "section", ]
        caption = ["table", ]
        dfn = ["dfn", ]
        dt = ["article", "aside", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "nav", "section", ]
        label = ["label", ]
        meter = ["meter", ]
        noscript = ["noscript", ]
        progress = ["progress", ]
        th = ["article", "aside", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "nav", "section", ]
    "#;

    #[test]
    fn good_case() {
        test_case("<nav><address>Hello</address></nav>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
        <address><nav>Hello</nav></address>
                 -----     ------
                 descendant_blacklist: `nav` not allowed within `address`
        "#,
            CONFIG,
            &registry(),
        )
    }
}
