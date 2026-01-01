use htmlsnob::ast::{CloseTag, Node, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;

/// Enforces that an elements are not too deeply nested.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    maximum_depth: usize,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
}

fn default_error_message() -> String {
    "Element is nested too deeply (maximum depth is {maximum_depth})".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        let tag = match (open_tag, close_tag) {
            (Some(ot), None) => ot,
            (None, Some(ct)) => match &parse_state.ast[ct.open_tag_index?] {
                Node::OpenTag(ot) => ot,
                _ => return None,
            },
            _ => return None,
        };

        if parse_state.open_tag_indexes.len() > self.maximum_depth {
            let message = dynamic_format(
                &self.error_message,
                &[("maximum_depth", self.maximum_depth.to_string())],
            );

            return Some(Warning::from_area(
                &self.name,
                &self.kind,
                tag.area.clone(),
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
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "maximum_nesting_depth"
        maximum_depth = 3
    "#;

    #[test]
    fn good_case() {
        test_case(
            r#"
            <div>
                <section>
                    <article>
                        <p>Valid nesting</p>
                    </article>
                </section>
            </div>
            "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case() {
        test_case(
            r#"
            <div>
                <section>
                    <article>
                        <div>
                            <p>Too deeply nested</p>
                        </div>
                    </article>
                </section>
            </div>
            "#,
            CONFIG,
            &registry(),
        )
    }
}
