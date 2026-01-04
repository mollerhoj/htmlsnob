use regex::Regex;
use serde::Deserialize;

use crate::ast::Construct;

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemplateLanguage {
    #[default]
    None,
    Eex,
    Handlebars,
    Jinja2,
    Liquid,
    Mustache,
    Erb,  // Not supported yet
    Go,   // Not supported yet
    Twig, // Not supported yet
}

impl TemplateLanguage {
    pub fn supports_quoting(&self) -> bool {
        matches!(
            self,
            TemplateLanguage::Handlebars
                | TemplateLanguage::Jinja2
                | TemplateLanguage::Liquid
                | TemplateLanguage::Mustache
                | TemplateLanguage::Twig
        )
    }
    pub fn from_filename(filename: &str) -> Self {
        if filename.ends_with(".eex") {
            TemplateLanguage::Eex
        } else if filename.ends_with(".erb") {
            TemplateLanguage::Erb
        } else if filename.ends_with(".gohtml") || filename.ends_with(".go") {
            TemplateLanguage::Go
        } else if filename.ends_with(".hbs") || filename.ends_with(".handlebars") {
            TemplateLanguage::Handlebars
        } else if filename.ends_with(".jinja2") || filename.ends_with(".jinja") {
            TemplateLanguage::Jinja2
        } else if filename.ends_with(".liquid") {
            TemplateLanguage::Liquid
        } else if filename.ends_with(".mustache") {
            TemplateLanguage::Mustache
        } else if filename.ends_with(".twig") {
            TemplateLanguage::Twig
        } else {
            TemplateLanguage::None
        }
    }

    pub fn constructs(
        template_language: &TemplateLanguage,
    ) -> Vec<(Regex, Regex, Regex, Construct)> {
        match template_language {
            TemplateLanguage::None => vec![],
            TemplateLanguage::Handlebars => vec![
                group("{{#if", "{{", "}}", Construct::If),
                group("{{else", "{{", "}}", Construct::Else),
                group("{{/if", "{{", "}}", Construct::EndIf),
                group("{{#each", "{{", "}}", Construct::Loop),
                group("{{/each", "{{", "}}", Construct::EndLoop),
                group("{{^", "{{", "}}", Construct::Block),
                group("{{#", "{{", "}}", Construct::Block),
                group("{{/", "{{", "}}", Construct::EndBlock),
                group("{{{", "{{{", "}}}", Construct::Expression),
                group("{{", "{{", "}}", Construct::Expression),
            ],
            TemplateLanguage::Jinja2 => vec![
                group("{% if", "{%", "%}", Construct::If),
                group("{%- if", "{%", "%}", Construct::If),
                group("{% else", "{%", "%}", Construct::Else),
                group("{%- else", "{%", "%}", Construct::Else),
                group("{% elif", "{%", "%}", Construct::Else),
                group("{%- elif", "{%", "%}", Construct::Else),
                group("{% for", "{%", "%}", Construct::Loop),
                group("{%- for", "{%", "%}", Construct::Loop),
                group("{% block", "{%", "%}", Construct::Block),
                group("{%- block", "{%", "%}", Construct::Block),
                group("{% raw", "{%", "%}", Construct::Block),
                group("{%- raw", "{%", "%}", Construct::Block),
                group("{% macro", "{%", "%}", Construct::Block),
                group("{%- macro", "{%", "%}", Construct::Block),
                group("{% with", "{%", "%}", Construct::Block),
                group("{%- with", "{%", "%}", Construct::Block),
                group("{% endif", "{%", "%}", Construct::EndIf),
                group("{%- endif", "{%", "%}", Construct::EndIf),
                group("{% endfor", "{%", "%}", Construct::EndLoop),
                group("{%- endfor", "{%", "%}", Construct::EndLoop),
                group("{% endblock", "{%", "%}", Construct::EndBlock),
                group("{%- endblock", "{%", "%}", Construct::EndBlock),
                group("{% endraw", "{%", "%}", Construct::EndBlock),
                group("{%- endraw", "{%", "%}", Construct::EndBlock),
                group("{% endmacro", "{%", "%}", Construct::EndBlock),
                group("{%- endmacro", "{%", "%}", Construct::EndBlock),
                group("{% endwith", "{%", "%}", Construct::EndBlock),
                group("{%- endwith", "{%", "%}", Construct::EndBlock),
                group("{{", "{{", "}}", Construct::Expression),
                group("{%", "{%", "%}", Construct::Statement),
                group("{#", "{#", "#}", Construct::Comment),
            ],
            TemplateLanguage::Liquid => vec![
                group("{% if", "{%", "%}", Construct::If),
                group("{%- if", "{%", "%}", Construct::If),
                group("{% else", "{%", "%}", Construct::Else),
                group("{%- else", "{%", "%}", Construct::Else),
                group("{% elsif", "{%", "%}", Construct::Else),
                group("{%- elsif", "{%", "%}", Construct::Else),
                group("{% for", "{%", "%}", Construct::Loop),
                group("{%- for", "{%", "%}", Construct::Loop),
                group("{% case", "{%", "%}", Construct::Switch),
                group("{%- case", "{%", "%}", Construct::Switch),
                group("{% when", "{%", "%}", Construct::Case),
                group("{%- when", "{%", "%}", Construct::Case),
                group("{% raw", "{%", "%}", Construct::Block),
                group("{%- raw", "{%", "%}", Construct::Block),
                group("{% comment", "{%", "%}", Construct::Block),
                group("{%- comment", "{%", "%}", Construct::Block),
                group("{% capture", "{%", "%}", Construct::Block),
                group("{%- capture", "{%", "%}", Construct::Block),
                group("{% endif", "{%", "%}", Construct::EndIf),
                group("{%- endif", "{%", "%}", Construct::EndIf),
                group("{% endfor", "{%", "%}", Construct::EndLoop),
                group("{%- endfor", "{%", "%}", Construct::EndLoop),
                group("{% endcase", "{%", "%}", Construct::EndSwitch),
                group("{%- endcase", "{%", "%}", Construct::EndSwitch),
                group("{% endraw", "{%", "%}", Construct::EndBlock),
                group("{%- endraw", "{%", "%}", Construct::EndBlock),
                group("{% endcomment", "{%", "%}", Construct::EndBlock),
                group("{%- endcomment", "{%", "%}", Construct::EndBlock),
                group("{% endcapture", "{%", "%}", Construct::EndBlock),
                group("{%- endcapture", "{%", "%}", Construct::EndBlock),
                group("{{", "{{", "}}", Construct::Expression),
                group("{%", "{%", "%}", Construct::Statement),
            ],
            TemplateLanguage::Mustache => vec![
                group("{{#", "{{", "}}", Construct::Block),
                group("{{^", "{{", "}}", Construct::Block),
                group("{{/", "{{", "}}", Construct::EndBlock),
                group("{{{", "{{{", "}}}", Construct::Expression),
                group("{{", "{{", "}}", Construct::Expression),
            ],
            TemplateLanguage::Twig => vec![
                group("{% if", "{%", "%}", Construct::If),
                group("{%- if", "{%", "%}", Construct::If),
                group("{% else", "{%", "%}", Construct::Else),
                group("{%- else", "{%", "%}", Construct::Else),
                group("{% elseif", "{%", "%}", Construct::Else),
                group("{%- elseif", "{%", "%}", Construct::Else),
                group("{% endif", "{%", "%}", Construct::EndIf),
                group("{%- endif", "{%", "%}", Construct::EndIf),
                group("{% for", "{%", "%}", Construct::Loop),
                group("{%- for", "{%", "%}", Construct::Loop),
                group("{% endfor", "{%", "%}", Construct::EndLoop),
                group("{%- endfor", "{%", "%}", Construct::EndLoop),
                group("{% block", "{%", "%}", Construct::Block),
                group("{%- block", "{%", "%}", Construct::Block),
                group("{% endblock", "{%", "%}", Construct::EndBlock),
                group("{%- endblock", "{%", "%}", Construct::EndBlock),
                group("{% macro", "{%", "%}", Construct::Block),
                group("{%- macro", "{%", "%}", Construct::Block),
                group("{% endmacro", "{%", "%}", Construct::EndBlock),
                group("{%- endmacro", "{%", "%}", Construct::EndBlock),
                group("{% raw", "{%", "%}", Construct::Block),
                group("{%- raw", "{%", "%}", Construct::Block),
                group("{% endraw", "{%", "%}", Construct::EndBlock),
                group("{%- endraw", "{%", "%}", Construct::EndBlock),
                group("{% embed", "{%", "%}", Construct::Block),
                group("{%- embed", "{%", "%}", Construct::Block),
                group("{% endembed", "{%", "%}", Construct::EndBlock),
                group("{%- endembed", "{%", "%}", Construct::EndBlock),
                group("{% verbatim", "{%", "%}", Construct::Block),
                group("{%- verbatim", "{%", "%}", Construct::Block),
                group("{% endverbatim", "{%", "%}", Construct::EndBlock),
                group("{%- endverbatim", "{%", "%}", Construct::EndBlock),
                group("{% filter", "{%", "%}", Construct::Block),
                group("{%- filter", "{%", "%}", Construct::Block),
                group("{% endfilter", "{%", "%}", Construct::EndBlock),
                group("{%- endfilter", "{%", "%}", Construct::EndBlock),
                group("{% spaceless", "{%", "%}", Construct::Block),
                group("{%- spaceless", "{%", "%}", Construct::Block),
                group("{% endspaceless", "{%", "%}", Construct::EndBlock),
                group("{%- endspaceless", "{%", "%}", Construct::EndBlock),
                group("{{", "{{", "}}", Construct::Expression),
                group("{%", "{%", "%}", Construct::Statement),
                group("{#", "{#", "#}", Construct::Comment),
            ],
            TemplateLanguage::Eex => vec![
                group("<% if", "<%", "%>", Construct::If),
                group("<%= if", "<%", "%>", Construct::If),
                group("<% unless", "<%", "%>", Construct::If),
                group("<%= unless", "<%", "%>", Construct::If),
                group("<% else", "<%", "%>", Construct::Else),
                group("<% elsif", "<%", "%>", Construct::Else),
                group("<% for", "<%", "%>", Construct::Loop),
                group("<%= for", "<%", "%>", Construct::Loop),
                group("<% case", "<%", "%>", Construct::Block), // hack as we can't detect EndCase
                group("<%= case", "<%", "%>", Construct::Block), // hack as we can't detect EndCase
                group("<% do ", "<%", "%>", Construct::Block),
                group("<%= do ", "<%", "%>", Construct::Block),
                group("<% begin", "<%", "%>", Construct::Block),
                group("<%= begin", "<%", "%>", Construct::Block),
                group("<% rescue", "<%", "%>", Construct::Else),
                group("<%= rescue", "<%", "%>", Construct::Else),
                group("<% ensure", "<%", "%>", Construct::Else),
                group("<%= case", "<%", "%>", Construct::Block),
                group("<% cond", "<%", "%>", Construct::Switch),
                group("<%= cond", "<%", "%>", Construct::Switch),
                group("<% end", "<%", "%>", Construct::EndBlock),
                group("<%=", "<%", "%>", Construct::Expression),
                group("<%==", "<%", "%>", Construct::Expression),
                group("<%", "<%", "%>", Construct::Statement),
            ],
            TemplateLanguage::Erb => vec![
                // TODO: Handle ERB constructs more accurately to allow indentation
                // TODO: Add <%- and -%> variants as well
                group("<% if", "<%", "%>", Construct::If),
                group("<% unless", "<%", "%>", Construct::If),
                group("<% while", "<%", "%>", Construct::Block),
                group("<% until", "<%", "%>", Construct::Block),
                group("<% else", "<%", "%>", Construct::Else),
                group("<% elsif", "<%", "%>", Construct::Else),
                group("<% end", "<%", "%>", Construct::EndBlock),
                group("<% for", "<%", "%>", Construct::Loop),
                group("<% case", "<%", "%>", Construct::Expression), // Hack as we can't detect EndCase
                group("<% when", "<%", "%>", Construct::Case),
                group("<% do", "<%", "%>", Construct::Block),
                group("<% begin", "<%", "%>", Construct::Block),
                group("<% rescue", "<%", "%>", Construct::Else),
                group("<% ensure", "<%", "%>", Construct::Else),
                group("<% end", "<%", "%>", Construct::EndBlock),
                group("<%=", "<%", "%>", Construct::Expression),
                group(r"<%\.\+ do \*|\.\+| \*%>", "<%", "%>", Construct::Block), // <% ... do |x| %>
                group(r"<%\.\+ do \*%>", "<%", "%>", Construct::Block),          // <% ... do %>
                group("<%", "<%", "%>", Construct::Statement),
            ],
            TemplateLanguage::Go => vec![
                group("{{if", "{{", "}}", Construct::If),
                group("{{else", "{{", "}}", Construct::Else),
                group("{{else if", "{{", "}}", Construct::Else),
                group("{{end", "{{", "}}", Construct::EndBlock),
                group("{{range", "{{", "}}", Construct::Loop),
                group("{{with", "{{", "}}", Construct::Block),
                group("{{template", "{{", "}}", Construct::Block),
                group("{{block", "{{", "}}", Construct::Block),
                group("{{define", "{{", "}}", Construct::Block),
                group("{{call", "{{", "}}", Construct::Block),
                group("{{", "{{", "}}", Construct::Expression),
                group("{{/*", "{{/*", "*/}}", Construct::Comment),
            ],
        }
    }
}

fn group(s1: &str, s2: &str, s3: &str, construct: Construct) -> (Regex, Regex, Regex, Construct) {
    (
        Regex::new(&format!(r"^{}", compile_bre_style(s1))).unwrap(),
        Regex::new(&format!(r"^{}", compile_bre_style(s2))).unwrap(),
        Regex::new(&format!(r"^{}", compile_bre_style(s3))).unwrap(),
        construct,
    )
}

fn compile_bre_style(pattern: &str) -> Regex {
    let escaped = regex::escape(pattern);

    let custom_pattern = escaped
        .replace(r"\\\.", ".")
        .replace(r"\\\*", "*")
        .replace(r"\\\+", "+");

    Regex::new(&custom_pattern).unwrap()
}
