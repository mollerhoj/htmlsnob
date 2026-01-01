use htmlsnob::ast::{CloseTag, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::fmt;

/// The casing style for tag names.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagNameCaseStyle {
    Lower,
    Upper,
}

impl TagNameCaseStyle {
    pub fn convert(&self, input: &str) -> String {
        match self {
            TagNameCaseStyle::Lower => input.to_lowercase(),
            TagNameCaseStyle::Upper => input.to_uppercase(),
        }
    }
}

impl fmt::Display for TagNameCaseStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TagNameCaseStyle::Lower => write!(f, "lowercase"),
            TagNameCaseStyle::Upper => write!(f, "uppercase"),
        }
    }
}

/// Enforces that all tag names match a specified casing style.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    /// If true, the rule will automatically convert to the preferred case style
    #[serde(default)]
    pub autofix: bool,
    /// The desired casing style for tag names.
    pub style: TagNameCaseStyle,
    /// The error message to display when the rule fails.
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
}

fn default_error_message() -> String {
    "Tag name \"{name}\" should be in {style}, change to \"{converted_name}\"".to_string()
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        let converted_name = self.style.convert(&open_tag.name);

        if open_tag.name == converted_name {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", open_tag.name.clone()),
                ("style", self.style.to_string()),
                ("converted_name", converted_name.clone()),
            ],
        );

        let warning = Some(Warning::from_area(
            &self.name,
            &self.kind,
            open_tag.area.clone(),
            &message,
            self.severity.clone(),
        ));

        if self.autofix {
            open_tag.name = converted_name;
        }

        warning
    }

    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        let converted_name = self.style.convert(&close_tag.name);

        if close_tag.name == converted_name {
            return None;
        }

        let message = dynamic_format(
            &self.error_message,
            &[
                ("name", close_tag.name.clone()),
                ("style", self.style.to_string()),
                ("converted_name", converted_name.clone()),
            ],
        );

        let warning = Some(Warning::from_area(
            &self.name,
            &self.kind,
            close_tag.area.clone(),
            &message,
            self.severity.clone(),
        ));

        if self.autofix {
            close_tag.name = converted_name;
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
        kind = "tag_name_casing"
        style = "lower"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<div><p>Hello</p></div>", CONFIG, &registry())
    }

    #[test]
    fn bad_case_opening_tag() {
        test_case_autofix(
            r#"
            <DIV>Hello</DIV>
            -----     ------
                      tag_name_casing: Tag name "DIV" should be in lowercase, change to "div"
            "#,
            r#"<div>Hello</div>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_closing_tag() {
        test_case_autofix(
            r#"
            <div>Hello</DIV>
                      ------
                      tag_name_casing: Tag name "DIV" should be in lowercase, change to "div"
            "#,
            r#"<div>Hello</div>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_both_tags() {
        test_case_autofix(
            r#"
            <DIV>Hello</DIV>
            -----     ------
                      tag_name_casing: Tag name "DIV" should be in lowercase, change to "div"
            "#,
            r#"<div>Hello</div>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn upper_case_style() {
        const UPPER_CONFIG: &str = r#"
            [[rules]]
            kind = "tag_name_casing"
            style = "upper"
            autofix = true
        "#;

        test_case_autofix(
            r#"
            <div>Hello</div>
            -----     ------
                      tag_name_casing: Tag name "div" should be in uppercase, change to "DIV"
            "#,
            r#"<DIV>Hello</DIV>"#,
            UPPER_CONFIG,
            &registry(),
        )
    }
}
