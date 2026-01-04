use crate::ast::Area;
use crate::ast::Attribute;
use crate::ast::AttributeValue;
use crate::ast::CloseTag;
use crate::ast::Comment;
use crate::ast::Construct;
use crate::ast::Doctype;
use crate::ast::Either;
use crate::ast::Node;
use crate::ast::OpenTag;
use crate::ast::Position;
use crate::ast::StringArea;
use crate::ast::TemplateExpression;
use crate::ast::Text;
use crate::config::Config;
use crate::template_language::TemplateLanguage;
use crate::warning::Warning;

const RAW_TEXT_ELEMENTS: [&str; 3] = ["script", "style", "textrange"];

pub struct Parser {
    template_language: TemplateLanguage,
    input: Vec<char>,
    line_number: usize,
    line_start_cursor: usize,
    cursor: usize,
    ignore_rules: bool,
    pub state: ParseState,
    pub warnings: Vec<Warning>,
}

pub struct ParseState {
    pub ast: Vec<Node>,
    pub open_tag_indexes: Vec<usize>,
    pub raw_text_mode_end_tag_name: Option<String>, // End tag in raw mode, e.g. "script"
}

impl ParseState {
    fn new() -> Self {
        ParseState {
            ast: Vec::new(),
            open_tag_indexes: Vec::new(),
            raw_text_mode_end_tag_name: None,
        }
    }

    pub fn open_tag_names(&self) -> Vec<String> {
        self.open_tag_indexes
            .iter()
            .map(|&index| match &self.ast[index] {
                Node::OpenTag(open_tag) => open_tag.name.clone(),
                _ => panic!("Expected an OpenTag"),
            })
            .collect()
    }
}

impl Parser {
    pub fn new(input: &str, config: &Config) -> Self {
        Parser {
            state: ParseState::new(),
            template_language: config.options.template_language.clone(),
            input: input.chars().collect(),
            line_number: 0,
            line_start_cursor: 0,
            cursor: 0,
            ignore_rules: false,
            warnings: Vec::new(),
        }
    }

    pub fn next_node(&mut self) -> Option<Node> {
        self.skip_whitespace();
        if self.cursor >= self.input.len() {
            return None;
        }

        Some(if self.state.raw_text_mode_end_tag_name.is_some() {
            let raw_text = self.parse_raw_text();
            self.state.raw_text_mode_end_tag_name = None;
            Node::Text(raw_text)
        } else if self.peek_str("</") {
            Node::CloseTag(self.parse_close_tag())
        } else if self.peek_str("<!--") {
            Node::Comment(self.parse_comment())
        } else if self.peek_str("<!DOCTYPE") || self.peek_str("<!doctype") {
            Node::Doctype(self.parse_doctype())
        } else if self.is_template_expression() {
            Node::TemplateExpression(self.parse_template_expression())
        } else if self.peek_char('<') {
            let open_tag = self.parse_open_tag();

            // Enter raw text mode if applicable
            if RAW_TEXT_ELEMENTS.contains(&open_tag.name.as_str()) && !open_tag.self_closed {
                self.state.raw_text_mode_end_tag_name = Some(open_tag.name.clone());
            }

            Node::OpenTag(open_tag)
        } else {
            Node::Text(self.parse_text())
        })
    }

    pub fn add_node(&mut self, node: Node) {
        // Track open tags
        if let Node::OpenTag(open_tag) = &node {
            if !open_tag.self_closed {
                self.state.open_tag_indexes.push(open_tag.index);
            }
        }

        self.state.ast.push(node);
    }

    fn parse_comment(&mut self) -> Comment {
        let start_position = self.position();
        self.consume_str("<!--");

        let start = self.cursor;

        // Find the end of the comment
        while self.cursor < self.input.len() {
            if self.peek_str("ignore above") {
                self.ignore_rules = false;
            }
            if self.peek_str("ignore below") {
                self.ignore_rules = true;
            }

            if self.peek_str("-->") {
                break;
            }
            self.advance();
        }

        let content: String = self.input[start..self.cursor].iter().collect();

        let mut is_missing_end_bracket = false;
        if self.peek_str("-->") {
            self.consume_str("-->");
        } else {
            is_missing_end_bracket = true;
        }

        Comment {
            content,
            area: Area {
                start: start_position,
                end: self.position(),
            },
            is_missing_end_bracket,
        }
    }

    fn position(&self) -> Position {
        Position {
            line: self.line_number,
            column: self.column_number(),
        }
    }

    fn parse_open_tag(&mut self) -> OpenTag {
        let start_position = self.position();
        // Parse the opening tag
        self.consume('<');
        self.skip_whitespace();

        let (name, is_missing_end_bracket) = self.parse_tag_name();
        let attributes = self.parse_attributes();

        // Self-closing tag
        if self.peek_str("/>") {
            self.consume_str("/>");

            return OpenTag {
                name: name.clone(),
                attributes: attributes.clone(),
                area: Area {
                    start: start_position.clone(),
                    end: self.position(),
                },
                self_closed: true,
                is_missing_end_bracket: false,
                close_tag_index: None, // To be filled when matching close tag is found
                index: self.state.ast.len(),
            };
        }

        if !is_missing_end_bracket {
            self.consume('>');
        }
        let end_position = self.position();

        OpenTag {
            name,
            attributes,
            area: Area {
                start: start_position,
                end: end_position,
            },
            self_closed: false,
            is_missing_end_bracket,
            close_tag_index: None, // Will be set *if* a matching close tag is found
            index: self.state.ast.len(),
        }
    }

    fn parse_close_tag(&mut self) -> CloseTag {
        let start_position = self.position();
        self.consume('<');
        self.consume('/');
        self.skip_whitespace();
        let (name, is_missing_end_bracket) = self.parse_tag_name();
        if !is_missing_end_bracket {
            self.consume('>');
        }
        let end_position = self.position();

        // Check if a matching opening tag exists
        let matched = self.state.open_tag_indexes.iter().any(|&open_tag_index| {
            let open_tag = match &self.state.ast[open_tag_index] {
                Node::OpenTag(open_tag) => open_tag,
                _ => panic!("Expected an OpenElement"),
            };
            open_tag.name == name
        });

        // Orphan closing tag
        if !matched {
            return CloseTag {
                name,
                area: Area {
                    start: start_position,
                    end: end_position,
                },
                is_missing_end_bracket,
                open_tag_index: None,
            };
        }

        // Since a match exists:
        // Traverse the stack to find the matching opening tag, marking passed elements as unclosed
        let index = self.state.ast.len();
        while let Some(open_tag_index) = self.state.open_tag_indexes.pop() {
            let open_tag = match &self.state.ast[open_tag_index] {
                Node::OpenTag(element) => element,
                _ => panic!("Expected an OpenTag"),
            };

            if open_tag.name == name {
                // Update OpenTag
                self.state.ast[open_tag_index] = Node::OpenTag(OpenTag {
                    name: open_tag.name.clone(),
                    attributes: open_tag.attributes.clone(),
                    area: open_tag.area.clone(),
                    self_closed: open_tag.self_closed,
                    is_missing_end_bracket: open_tag.is_missing_end_bracket,
                    close_tag_index: Some(index),
                    index: open_tag.index,
                });

                // Return CloseTag
                return CloseTag {
                    name,
                    area: Area {
                        start: start_position,
                        end: end_position,
                    },
                    is_missing_end_bracket,
                    open_tag_index: Some(open_tag_index),
                };
            }
        }

        // No matching opening tag found
        CloseTag {
            name,
            area: Area {
                start: start_position,
                end: end_position,
            },
            is_missing_end_bracket,
            open_tag_index: None,
        }
    }

    fn column_number(&self) -> usize {
        self.cursor - self.line_start_cursor
    }

    fn parse_raw_text(&mut self) -> Text {
        let start_position = self.position();
        let start = self.cursor;
        let end_tag = format!(
            "</{}",
            self.state.raw_text_mode_end_tag_name.as_ref().unwrap()
        );

        while self.cursor < self.input.len() && !self.peek_str(&end_tag) {
            self.advance();
        }
        let content = self.input[start..self.cursor]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();

        Text {
            content,
            area: Area {
                start: start_position,
                end: self.position(),
            },
        }
    }

    fn parse_tag_name(&mut self) -> (String, bool) {
        let start = self.cursor;

        while self.cursor < self.input.len()
            && !self.is_whitespace(self.current_char())
            && !self.peek_char('>')
            && !self.peek_str("/>")
        {
            if self.current_char() == '<' {
                let name: String = self.input[start..self.cursor].iter().collect();
                return (name, true);
            }

            self.advance();
        }

        let name: String = self.input[start..self.cursor].iter().collect();
        (name, self.cursor == self.input.len())
    }

    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attributes = Vec::new();
        self.skip_whitespace();
        while self.cursor < self.input.len() && !self.peek_char('>') && !self.peek_str("/>") {
            let start_position = self.position();
            let name = self.parse_attribute_name();

            // Handle boolean attributes (without value)
            self.skip_whitespace();
            let mut value = None;
            if self.peek_char('=') {
                self.consume('=');
                self.skip_whitespace();
                value = Some(self.parse_attribute_value());
            }

            attributes.push(Attribute {
                name,
                value,
                area: Area {
                    start: start_position,
                    end: self.position(),
                },
            });

            self.skip_whitespace();
        }

        attributes
    }

    fn parse_attribute_name(&mut self) -> Either<StringArea, TemplateExpression> {
        if self.is_template_expression() {
            return Either::Right(self.parse_template_expression());
        }

        let start_cursor = self.cursor;
        let start_position = self.position();

        while self.cursor < self.input.len()
            && !self.is_whitespace(self.current_char())
            && !self.peek_any(&["=", ">", "/>", " ", "\t", "\n", "\r"])
        {
            self.advance();
        }

        Either::Left(StringArea {
            content: self.input[start_cursor..self.cursor].iter().collect(),
            area: Area {
                start: start_position,
                end: self.position(),
            },
        })
    }

    fn parse_attribute_value(&mut self) -> AttributeValue {
        let start_position = self.position();
        let mut start_quote = None;
        if self.peek_char('\'') || self.peek_char('"') {
            let quote_char = self.current_char();
            start_quote = Some(quote_char);
            self.consume(quote_char);
        }

        let mut end_quote = None;
        let mut classes = Vec::new();
        let mut class_start = self.cursor;
        let mut class_start_position = self.position();

        loop {
            if let Some(quote) = start_quote {
                // Parse quoted attribute value
                if self.peek_char(quote) {
                    end_quote = Some(quote);
                    if class_start < self.cursor {
                        classes.push(Either::Left(StringArea {
                            content: self.input[class_start..self.cursor]
                                .iter()
                                .collect::<String>(),
                            area: Area {
                                start: class_start_position,
                                end: self.position(),
                            },
                        }));
                    }
                    break;
                }

                if self.is_template_expression() {
                    if class_start < self.cursor {
                        classes.push(Either::Left(StringArea {
                            content: self.input[class_start..self.cursor]
                                .iter()
                                .collect::<String>(),
                            area: Area {
                                start: class_start_position,
                                end: self.position(),
                            },
                        }));
                    }
                    classes.push(Either::Right(self.parse_template_expression()));
                    class_start = self.cursor;
                    class_start_position = self.position();
                    continue;
                }

                if self.is_whitespace(self.current_char()) {
                    if class_start < self.cursor {
                        classes.push(Either::Left(StringArea {
                            content: self.input[class_start..self.cursor]
                                .iter()
                                .collect::<String>(),
                            area: Area {
                                start: class_start_position,
                                end: self.position(),
                            },
                        }));
                    }
                    self.advance(); // Skip whitespace
                    class_start = self.cursor;
                    class_start_position = self.position();
                    continue;
                }
            } else if self.is_whitespace(self.current_char())
                || self.peek_char('>')
                || self.peek_str("/>")
            {
                // Parse unquoted attribute value
                classes.push(Either::Left(StringArea {
                    content: self.input[class_start..self.cursor]
                        .iter()
                        .collect::<String>(),
                    area: Area {
                        start: class_start_position,
                        end: self.position(),
                    },
                }));
                break;
            }

            if self.cursor >= self.input.len() {
                panic!("Unexpected end of input");
            }

            self.advance();
        }

        if let Some(quote) = end_quote {
            self.consume(quote);
        }

        AttributeValue {
            area: Area {
                start: start_position,
                end: self.position(),
            },
            parts: classes,
            start_quote,
            end_quote,
        }
    }

    fn parse_text(&mut self) -> Text {
        let start = self.cursor;
        let start_position = self.position();

        while self.cursor < self.input.len()
            && (self.peek_str("<<") || !self.peek_char('<'))
            && !self.is_template_expression()
        {
            self.advance();
        }

        let text: String = self.input[start..self.cursor].iter().collect();
        let text = text.trim().to_string();

        Text {
            content: text,
            area: Area {
                start: start_position,
                end: self.position(),
            },
        }
    }

    fn parse_doctype(&mut self) -> Doctype {
        let start_position = self.position();
        self.consume_str("<!");
        self.skip_whitespace();

        let start = self.cursor;

        // Find the end of the doctype
        while self.cursor < self.input.len() {
            if self.peek_char('>') {
                break;
            }
            self.advance();
        }

        // Extract the doctype content
        let content: String = self.input[start..self.cursor].iter().collect();

        let mut is_missing_end_bracket = false;
        if self.peek_char('>') {
            self.consume('>');
        } else {
            is_missing_end_bracket = true;
        }

        Doctype {
            content,
            area: Area {
                start: start_position,
                end: self.position(),
            },
            is_missing_end_bracket,
        }
    }

    fn is_template_expression(&self) -> bool {
        if self.cursor > 0
            // Handlebars escape character
            && (self.input[self.cursor - 1] == '\\' || self.input[self.cursor - 1] == '{')
        {
            return false;
        }

        TemplateLanguage::constructs(&self.template_language)
            .into_iter()
            .any(|construct| self.peek_regex(&construct.0))
    }

    fn parse_template_expression(&mut self) -> TemplateExpression {
        let start_position = self.position();
        let content_start = self.cursor;

        let construct = TemplateLanguage::constructs(&self.template_language)
            .into_iter()
            .find(|construct| self.peek_regex(&construct.0))
            .unwrap_or_else(|| panic!("No matching construct found for input"));

        let mut quote = None;
        let mut is_missing_end_bracket = false;
        loop {
            if self.cursor >= self.input.len() {
                is_missing_end_bracket = true;
                break;
            }

            let quoting_supported = self.template_language.supports_quoting();
            let escaped = self.cursor > 0 && self.input[self.cursor - 1] == '\\';
            if !escaped && quoting_supported && construct.3 != Construct::Comment {
                if quote.is_none() {
                    if self.peek_str("\"") {
                        quote = Some('"');
                    }

                    if self.peek_str("'") {
                        quote = Some('\'');
                    }
                } else if (quote == Some('\'') && self.peek_str("'"))
                    || (quote == Some('"') && self.peek_str("\""))
                {
                    quote = None;
                }
            }

            if quote.is_none() && self.peek_regex(&construct.2) {
                self.consume_regex(&construct.2);
                break;
            }

            self.advance();
        }

        TemplateExpression {
            content: self.input[content_start..self.cursor].iter().collect(),
            area: Area {
                start: start_position,
                end: self.position(),
            },
            kind: construct.3.clone(),
            is_missing_end_bracket,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.cursor < self.input.len() && self.is_whitespace(self.current_char()) {
            self.advance();
        }
    }

    fn is_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t' || c == '\n' || c == '\r'
    }

    fn current_char(&self) -> char {
        if self.cursor < self.input.len() {
            self.input[self.cursor]
        } else {
            '\0'
        }
    }

    fn peek_char(&self, expected: char) -> bool {
        self.cursor < self.input.len() && self.input[self.cursor] == expected
    }

    fn peek_str(&self, s: &str) -> bool {
        if self.cursor + s.len() > self.input.len() {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if self.input[self.cursor + i] != c {
                return false;
            }
        }
        true
    }

    fn peek_regex(&self, regex: &regex::Regex) -> bool {
        let remaining_input: String = self.input[self.cursor..].iter().collect();
        regex.is_match(&remaining_input)
    }

    fn consume_regex(&mut self, regex: &regex::Regex) {
        let remaining_input: String = self.input[self.cursor..].iter().collect();
        if let Some(regex_match) = regex.find(&remaining_input) {
            assert_eq!(
                regex_match.start(),
                0,
                "Regex did not match at the current position"
            );
            for _ in 0..regex_match.end() {
                self.advance();
            }
        } else {
            panic!("Regex did not match the input");
        }
    }

    fn peek_any(&self, list: &[&str]) -> bool {
        for &s in list {
            if self.peek_str(s) {
                return true;
            }
        }
        false
    }

    fn advance(&mut self) {
        if self.current_char() == '\n' {
            self.line_number += 1;
            self.line_start_cursor = self.cursor + 1;
        }
        self.cursor += 1;
    }

    fn consume(&mut self, expected: char) {
        assert_eq!(
            self.current_char(),
            expected,
            "Input: {}",
            self.input.iter().collect::<String>()
        );
        self.advance();
    }

    fn consume_str(&mut self, expected: &str) {
        assert!(self.peek_str(expected));
        for _ in 0..expected.len() {
            self.advance();
        }
    }

    #[allow(dead_code)]
    fn debug(&self, prefix: &str) {
        let string_before_cursor: String = self.input[..self.cursor].iter().collect();
        let string_after_cursor: String = self.input[self.cursor..].iter().collect();
        println!(
            "Context, {}: '{}|{}'",
            prefix, string_before_cursor, string_after_cursor
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Node;
    use crate::config::Config;
    use crate::parser::Parser;
    use crate::registry::Registry;

    #[test]
    fn test_parser() {
        let input = "yo<a><p>lo<XXX>Hello<i></XXX></a>";
        let registry = Registry::new();
        let config = Config::from_toml("", &registry);
        let mut parser = Parser::new(input, &config);

        while let Some(node) = parser.next_node() {
            parser.add_node(node);
        }

        for node in &parser.state.ast {
            match node {
                Node::OpenTag(open_tag) => {
                    println!(
                        "OT: {:?} {:?} {:?}",
                        open_tag.name, open_tag.self_closed, open_tag.close_tag_index
                    );
                }
                Node::CloseTag(close_tag) => {
                    println!("CT: {:?} {:?}", close_tag.name, close_tag.open_tag_index);
                }
                Node::Text(text) => {
                    println!("{:?}", text);
                }
                _ => {}
            }
        }

        let formatter = crate::formatter::Formatter::new(&parser.state.ast, &config);
        let output = formatter.format();
        println!("OUTPUT:\n{}", output);

        //panic!("TEST END");
    }
}
