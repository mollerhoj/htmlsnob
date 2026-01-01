pub mod ast;
pub mod case_converter;
pub mod config;
pub mod dynamic_format;
pub mod formatter;
pub mod parser;
pub mod registry;
pub mod rule_trait;
pub mod rule_utils;
pub mod template_language;
pub mod test_utils;
pub mod warning;

use ast::Node;
mod serde_expander;
mod validator;
use formatter::Formatter;
use parser::Parser;

use validator::Validator;
// RePublish the warning::Warning struct
pub use warning::Warning;
pub use warning::WarningSeverity;

pub fn lint(input: &str, config: &mut config::Config) -> (Vec<Node>, Vec<Warning>) {
    let mut parser = Parser::new(input, config);
    let mut validator = Validator::new(config);
    let mut warnings = Vec::new();
    while let Some(mut node) = parser.next_node() {
        warnings.extend(validator.validate(&mut node, &mut parser.state));
        parser.add_node(node);
    }
    warnings.extend(validator.finalize(&parser.state));

    (parser.state.ast, warnings)
}

pub fn format(ast: &Vec<Node>, config: &config::Config) -> String {
    Formatter::new(ast, config).format()
}
