use htmlsnob::ast::{Area, CloseTag, Comment, Doctype, OpenTag, TemplateExpression};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that tags are not missing their end bracket.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default)]
    pub autofix: bool,
}

fn default_error_message() -> String {
    "{name} is missing end bracket".to_string()
}

impl Rule {
    fn apply(&self, is_missing_end_bracket: &mut bool, area: &Area, name: &str) -> Option<Warning> {
        if !*is_missing_end_bracket {
            return None;
        }

        if self.autofix {
            *is_missing_end_bracket = false;
        }

        Some(Warning::from_area(
            &self.name,
            &self.kind,
            area.clone(),
            &dynamic_format(&self.error_message, &[("name", name.to_string())]),
            self.severity.clone(),
        ))
    }
}

impl RuleTrait for Rule {
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        self.apply(
            &mut open_tag.is_missing_end_bracket,
            &open_tag.area,
            &format!("Open tag `{}`", open_tag.name),
        )
    }

    fn apply_close_tag(
        &self,
        close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        self.apply(
            &mut close_tag.is_missing_end_bracket,
            &close_tag.area,
            &format!("Close tag `{}`", close_tag.name),
        )
    }

    fn apply_template_expression(
        &self,
        template_expression: &mut TemplateExpression,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        self.apply(
            &mut template_expression.is_missing_end_bracket,
            &template_expression.area,
            "Template expression",
        )
    }

    fn apply_doctype(&self, doctype: &mut Doctype, _parse_state: &ParseState) -> Option<Warning> {
        self.apply(
            &mut doctype.is_missing_end_bracket,
            &doctype.area,
            "Doctype",
        )
    }

    fn apply_comment(&self, comment: &mut Comment, _parse_state: &ParseState) -> Option<Warning> {
        self.apply(
            &mut comment.is_missing_end_bracket,
            &comment.area,
            "Comment",
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;
    use htmlsnob::test_utils::tests::test_case_autofix;

    const CONFIG: &str = r#"
        template_language = "handlebars"

        [[rules]]
        kind = "missing_end_bracket_disallowed"
        autofix = true
    "#;

    #[test]
    fn good_case() {
        test_case("<p></p>", CONFIG, &registry())
    }

    #[test]
    fn bad_case_open_tag() {
        test_case_autofix(
            r#"
            <p
            --
            missing_end_bracket_disallowed: Open tag `p` is missing end bracket
            "#,
            r#"<p>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_close_tag() {
        test_case_autofix(
            r#"
            <p></p
               ---
               missing_end_bracket_disallowed: Close tag `p` is missing end bracket
            "#,
            r#"<p></p>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_template_expression_not_autofixed() {
        test_case_autofix(
            r#"
            {{ if condition
            ---------------
            missing_end_bracket_disallowed: Template expression is missing end bracket
            "#,
            r#"{{ if condition"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_doctype() {
        test_case_autofix(
            r#"
            <!DOCTYPE html
            --------------
            missing_end_bracket_disallowed: Doctype is missing end bracket
            "#,
            r#"<!DOCTYPE html>"#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_comment() {
        test_case_autofix(
            r#"
            <!-- This is a comment
            ----------------------
            missing_end_bracket_disallowed: Comment is missing end bracket
            "#,
            r#"<!-- This is a comment -->"#,
            CONFIG,
            &registry(),
        )
    }
}
