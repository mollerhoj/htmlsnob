use crate::{ast::Node, config::Config, parser::ParseState, Warning};

pub struct Validator<'a> {
    config: &'a mut Config,
}

impl<'a> Validator<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Validator { config }
    }

    pub fn validate(&mut self, node: &mut Node, parse_state: &ParseState) -> Vec<Warning> {
        let warnings = self.apply(node, parse_state);
        self.track(node, parse_state);
        warnings
    }

    pub fn finalize(&mut self, parse_state: &ParseState) -> Vec<Warning> {
        let mut warnings = Vec::new();

        for open_tag_index in parse_state.open_tag_indexes.iter() {
            for rule in self.config.rules.iter_mut() {
                let open_tag = match &parse_state.ast[*open_tag_index] {
                    Node::OpenTag(open_tag) => open_tag,
                    _ => panic!("Expected OpenTag at index {}", open_tag_index),
                };

                warnings.extend(rule.apply_tag(Some(open_tag), None, parse_state));
            }
        }

        for rule in self.config.rules.iter_mut() {
            rule.reset_state();
        }

        warnings
    }

    fn apply(&mut self, node: &mut Node, parse_state: &ParseState) -> Vec<Warning> {
        let mut warnings = Vec::new();

        for rule in self.config.rules.iter_mut() {
            match node {
                Node::OpenTag(open_tag) => {
                    warnings.extend(rule.apply_open_tag(open_tag, parse_state));
                    warnings.extend(rule.apply_open_tag_vec(open_tag, parse_state));

                    for attribute in open_tag.attributes.iter_mut() {
                        warnings.extend(rule.apply_attribute(attribute));
                        warnings.extend(rule.apply_attribute_vec(attribute));
                    }
                }
                Node::CloseTag(close_tag) => {
                    let mut open_tag = None;
                    if let Some(open_tag_index) = close_tag.open_tag_index {
                        if let Node::OpenTag(ref open_tag_ref) = parse_state.ast[open_tag_index] {
                            open_tag = Some(open_tag_ref);
                        } else {
                            panic!("Expected OpenTag at index {}", open_tag_index);
                        }
                    }

                    warnings.extend(rule.apply_close_tag(close_tag, parse_state));
                    warnings.extend(rule.apply_close_tag_vec(close_tag, parse_state));
                    warnings.extend(rule.apply_tag(open_tag, Some(close_tag), parse_state));
                    warnings.extend(rule.apply_tag_vec(open_tag, Some(close_tag), parse_state));
                }
                Node::Text(text) => warnings.extend(rule.apply_text(text, parse_state)),
                Node::TemplateExpression(template_expression) => {
                    warnings
                        .extend(rule.apply_template_expression(template_expression, parse_state));
                    warnings.extend(
                        rule.apply_template_expression_vec(template_expression, parse_state),
                    );
                }
                Node::Doctype(doctype) => {
                    warnings.extend(rule.apply_doctype(doctype, parse_state));
                    warnings.extend(rule.apply_doctype_vec(doctype, parse_state));
                }
                Node::Comment(comment) => {
                    warnings.extend(rule.apply_comment(comment, parse_state));
                    warnings.extend(rule.apply_comment_vec(comment, parse_state));
                }
            };
        }

        warnings
    }

    fn track(&mut self, node: &Node, parse_state: &ParseState) {
        for rule in self.config.rules.iter_mut() {
            match node {
                Node::OpenTag(open_tag) => rule.track_open_tag(open_tag, parse_state),
                Node::CloseTag(close_tag) => rule.track_close_tag(close_tag, parse_state),
                Node::Text(text) => rule.track_text(text, parse_state),
                _ => {}
            }
        }
    }
}
