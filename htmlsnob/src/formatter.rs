use crate::{
    ast::{Attribute, AttributeValue, Either, Node},
    config::{self, Config},
};

pub struct Formatter<'a> {
    config: &'a config::Config,
    ast: &'a Vec<Node>,
}

struct FormatRunner<'a> {
    config: &'a config::Config,
    ast: &'a Vec<Node>,
    index: usize,
    indent_level: usize,
    result: String,
}

impl<'a> Formatter<'a> {
    pub fn new(ast: &'a Vec<Node>, config: &'a Config) -> Self {
        Formatter { ast, config }
    }

    pub fn format(self) -> String {
        let mut runner = FormatRunner {
            config: self.config,
            ast: self.ast,
            indent_level: 0,
            index: 0,
            result: String::new(),
        };
        runner.run();
        runner.result
    }
}

impl<'a> FormatRunner<'a> {
    fn run(&mut self) {
        while self.index < self.ast.len() {
            self.format();
            self.index += 1;
        }
    }

    fn indent(&mut self, str: String) {
        self.result.push_str(self.indentation_spaces().as_str());
        self.result.push_str(&str);
    }

    fn format(&mut self) {
        let node = &self.ast[self.index];
        match node {
            Node::OpenTag(open_tag) => {
                let mut result = String::new();
                // ------ Opening tag --------------------------------------------------
                result
                    .push_str(format!("{}<{}", self.indentation_spaces(), &open_tag.name).as_str());

                // Formatting Exception 1:
                // If the element has a lot of attributes, we should put each on a new line.
                let mut attributes = self.add_attributes(&open_tag.attributes, false);
                if attributes.len() + self.indentation_spaces().len()
                    > self.config.options.max_line_length
                {
                    attributes = self.add_attributes(&open_tag.attributes, true);
                }

                result.push_str(&attributes);

                if open_tag.self_closed {
                    result.push_str(" />");
                } else if !open_tag.is_missing_end_bracket {
                    result.push('>');
                }

                // Handle special cases:

                // Special-case 1: If there is no close tag, just add a newline and return.
                if open_tag.close_tag_index.is_none() {
                    self.result.push_str(&result);
                    self.result.push('\n');
                    return;
                }

                let child_count = open_tag.close_tag_index.unwrap() - self.index - 1;

                let close_tag = match &self.ast[open_tag.close_tag_index.unwrap()] {
                    Node::CloseTag(close_tag) => close_tag,
                    _ => panic!(
                        "Expected close tag at index {}",
                        open_tag.close_tag_index.unwrap()
                    ),
                };
                let end_tag_string = format!("</{}>", &close_tag.name);

                if child_count == 0 {
                    // Special-case 2: If there are no children, format inline.
                    if result.len() + end_tag_string.len() < self.config.options.max_line_length {
                        result.push_str(&format!("{}\n", &end_tag_string));
                        self.result.push_str(&result);
                        self.index = open_tag.close_tag_index.unwrap();
                        return;
                    }
                } else if child_count == 1 {
                    // Special-case 3: If the only child is a text node or template expression, and it is short
                    // enough, format it inline.
                    let first_child = self.index + 1;
                    let str_child = match &self.ast[first_child] {
                        Node::Text(text) => Some(&text.content),
                        Node::TemplateExpression(template_expression) => {
                            Some(&template_expression.content)
                        }
                        _ => None,
                    };

                    if let Some(str_child) = str_child {
                        if result.len() + str_child.len() + end_tag_string.len()
                            < self.config.options.max_line_length
                        {
                            result.push_str(&format!("{}{}\n", str_child, &end_tag_string));
                            self.result.push_str(&result);
                            self.index = open_tag.close_tag_index.unwrap();
                            return;
                        }
                    }
                }

                // No special-case: Indent and newline:
                if open_tag.close_tag_index.is_some() {
                    self.indent_level += 1;
                }

                self.result.push_str(&result);
                self.result.push('\n');
            }
            Node::CloseTag(close_tag) => {
                if close_tag.open_tag_index.is_some() {
                    self.indent_level = self.indent_level.saturating_sub(1);
                }
                self.indent(format!("</{}>\n", &close_tag.name));
            }
            Node::Text(text) => self.indent(format!("{}\n", text.content)),
            Node::Comment(comment) => {
                if comment.is_missing_end_bracket {
                    self.indent(format!("<!--{}\n", comment.content))
                } else {
                    self.indent(format!("<!--{}-->\n", comment.content))
                }
            }
            Node::Doctype(doctype) => {
                if doctype.is_missing_end_bracket {
                    self.indent(format!("<!{}\n", doctype.content))
                } else {
                    self.indent(format!("<!{}>\n", doctype.content))
                }
            }
            Node::TemplateExpression(expression) => {
                match expression.kind {
                    crate::ast::Construct::EndBlock
                    | crate::ast::Construct::EndIf
                    | crate::ast::Construct::EndLoop
                    | crate::ast::Construct::Case
                    | crate::ast::Construct::Else => {
                        self.indent_level = self.indent_level.saturating_sub(1);
                    }
                    crate::ast::Construct::EndSwitch => {
                        self.indent_level = self.indent_level.saturating_sub(2);
                    }
                    _ => {}
                }

                self.indent(format!("{}\n", expression.content));

                match expression.kind {
                    crate::ast::Construct::Block
                    | crate::ast::Construct::If
                    | crate::ast::Construct::Loop
                    | crate::ast::Construct::Case
                    | crate::ast::Construct::Else => {
                        self.indent_level += 1;
                    }
                    crate::ast::Construct::Switch => {
                        self.indent_level += 2;
                    }
                    _ => {}
                }
            }
        }
    }

    fn add_attribute(&mut self, attribute: &Attribute) -> String {
        let mut result = String::new();

        result.push_str(match &attribute.name {
            Either::Left(string_area) => &string_area.content,
            Either::Right(template_expression) => &template_expression.content,
        });

        if let Some(value) = &attribute.value {
            result.push('=');

            if let Some(start_quote) = value.start_quote {
                result.push(start_quote);
            }

            let mut attribute_value = self.add_attribute_value(value, false);

            if attribute_value.len() + self.indentation_spaces().len()
                > self.config.options.max_line_length
            {
                attribute_value = self.add_attribute_value(value, true);
            }

            result.push_str(&attribute_value);

            if let Some(end_quote) = value.end_quote {
                result.push(end_quote);
            }
        }

        result
    }

    fn add_attribute_value(&mut self, value: &AttributeValue, add_new_lines: bool) -> String {
        let mut result = String::new();
        for (i, class) in value.parts.iter().enumerate() {
            match class {
                crate::ast::Either::Left(string_area) => {
                    result.push_str(&string_area.content);
                }
                crate::ast::Either::Right(template_expression) => {
                    result.push_str(&template_expression.content);
                }
            }

            if i < value.parts.len() - 1 {
                let next_class = &value.parts[i + 1];

                let consequtive_string_areas = class.is_left() && next_class.is_left();
                if add_new_lines && !consequtive_string_areas {
                    match class {
                        crate::ast::Either::Left(_) => {}
                        crate::ast::Either::Right(te) => match te.kind {
                            crate::ast::Construct::Block
                            | crate::ast::Construct::If
                            | crate::ast::Construct::Loop
                            | crate::ast::Construct::Case
                            | crate::ast::Construct::Else => {
                                self.indent_level += 1;
                            }
                            crate::ast::Construct::Switch => {
                                self.indent_level += 2;
                            }
                            _ => {}
                        },
                    }

                    match next_class {
                        crate::ast::Either::Left(_) => {}
                        crate::ast::Either::Right(te) => match te.kind {
                            crate::ast::Construct::EndBlock
                            | crate::ast::Construct::EndIf
                            | crate::ast::Construct::EndLoop
                            | crate::ast::Construct::Else => {
                                self.indent_level -= 1;
                            }
                            crate::ast::Construct::EndSwitch => {
                                self.indent_level -= 2;
                            }
                            _ => {}
                        },
                    }

                    result.push('\n');
                    result.push_str(
                        &" ".repeat((1 + self.indent_level) * self.config.options.indent_size),
                    );
                } else {
                    result.push(' ');
                }
            }
        }
        result
    }

    fn add_attributes(
        &mut self,
        attributes: &[Attribute],
        new_line_after_attribute: bool,
    ) -> String {
        let mut result = String::new();

        for (index, attribute) in attributes.iter().enumerate() {
            if new_line_after_attribute && index > 0 {
                result.push_str(
                    &" ".repeat((1 + self.indent_level) * self.config.options.indent_size),
                );
            } else {
                result.push(' ');
            }

            result.push_str(&self.add_attribute(attribute));

            if new_line_after_attribute && index < attributes.len() - 1 {
                result.push('\n');
            }
        }

        result
    }

    fn indentation_spaces(&self) -> String {
        " ".repeat(self.indent_level * self.config.options.indent_size)
    }
}
