use htmlsnob::ast::OpenTag;
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an elements specfied attribute does not have any of the specified values.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    style: SelfClosingTagStyle,
    pub tags: Vec<String>,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default)]
    pub autofix: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfClosingTagStyle {
    Closed,
    Open,
}

impl std::fmt::Display for SelfClosingTagStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelfClosingTagStyle::Closed => write!(f, "closed"),
            SelfClosingTagStyle::Open => write!(f, "open"),
        }
    }
}

fn default_error_message() -> String {
    "Tag `{name}` must be {style} `{example}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        if !self.tags.contains(&open_tag.name) {
            return None;
        }

        match self.style {
            SelfClosingTagStyle::Closed if open_tag.self_closed => return None,
            SelfClosingTagStyle::Open if !open_tag.self_closed => return None,
            _ => {}
        }

        if self.autofix {
            match self.style {
                SelfClosingTagStyle::Closed => {
                    open_tag.self_closed = true;
                }
                SelfClosingTagStyle::Open => {
                    open_tag.self_closed = false;
                }
            }
        }

        let example = match self.style {
            SelfClosingTagStyle::Closed => format!("<{}/>", open_tag.name),
            SelfClosingTagStyle::Open => format!("<{}>", open_tag.name),
        };

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", open_tag.name.clone()),
                ("style", self.style.to_string()),
                ("example", example),
            ],
        );

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            open_tag.area.clone(),
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

    const CONFIG: &str = r#"
        [[rules]]
        kind = "self_closing_tag_style"
        style = "closed" 
        tags = ["br"]
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<br/>", CONFIG, &registry())
    }

    #[test]
    fn bad_case() {
        test_case_autofix(
            r#"
            <br>
            ----
            self_closing_tag_style: Tag `br` must be closed `<br/>`
        "#,
            r#"<br/>"#,
            CONFIG,
            &registry(),
        )
    }
}
