use crate::ast::{Attribute, CloseTag, OpenTag, Text};
use crate::parser::ParseState;
use crate::warning::Warning;
use std::fmt::Debug;

pub trait RuleTrait: Debug + Send + Sync {
    fn reset_state(&mut self) {}
    fn apply_tag(
        &self,
        _open_tag: Option<&OpenTag>,
        _close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_tag_vec(
        &self,
        _open_tag: Option<&OpenTag>,
        _close_tag: Option<&CloseTag>,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_open_tag(
        &self,
        _open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_open_tag_vec(
        &self,
        _open_tag: &mut OpenTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_close_tag(
        &self,
        _close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_close_tag_vec(
        &self,
        _close_tag: &mut CloseTag,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_template_expression(
        &self,
        _template_expression: &mut crate::ast::TemplateExpression,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_template_expression_vec(
        &self,
        _template_expression: &mut crate::ast::TemplateExpression,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_doctype(
        &self,
        _doctype: &mut crate::ast::Doctype,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_doctype_vec(
        &self,
        _doctype: &mut crate::ast::Doctype,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_comment(
        &self,
        _comment: &mut crate::ast::Comment,
        _parse_state: &ParseState,
    ) -> Option<Warning> {
        None
    }
    fn apply_comment_vec(
        &self,
        _comment: &mut crate::ast::Comment,
        _parse_state: &ParseState,
    ) -> Vec<Warning> {
        vec![]
    }
    fn apply_text(&self, _text: &mut Text, _parse_state: &ParseState) -> Option<Warning> {
        None
    }
    fn apply_text_vec(&self, _text: &mut Text, _parse_state: &ParseState) -> Vec<Warning> {
        vec![]
    }
    fn apply_attribute(&self, _attribute: &mut Attribute) -> Option<Warning> {
        None
    }
    fn apply_attribute_vec(&self, _attribute: &mut Attribute) -> Vec<Warning> {
        vec![]
    }
    fn track_open_tag(&mut self, _node: &OpenTag, _parse_state: &ParseState) {}
    fn track_close_tag(&mut self, _node: &CloseTag, _parse_state: &ParseState) {}
    fn track_text(&mut self, _text: &Text, _parse_state: &ParseState) {}
}
