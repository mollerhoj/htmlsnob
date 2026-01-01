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
    ) -> Vec<(&'static str, &'static str, &'static str, Construct)> {
        match template_language {
            TemplateLanguage::None => vec![],
            TemplateLanguage::Handlebars => vec![
                ("{{#if", "{{", "}}", Construct::If),
                ("{{else", "{{", "}}", Construct::Else),
                ("{{/if", "{{", "}}", Construct::EndIf),
                ("{{#each", "{{", "}}", Construct::Loop),
                ("{{/each", "{{", "}}", Construct::EndLoop),
                ("{{^", "{{", "}}", Construct::Block),
                ("{{#", "{{", "}}", Construct::Block),
                ("{{/", "{{", "}}", Construct::EndBlock),
                ("{{{", "{{{", "}}}", Construct::Expression),
                ("{{", "{{", "}}", Construct::Expression),
            ],
            TemplateLanguage::Jinja2 => vec![
                ("{% if", "{%", "%}", Construct::If),
                ("{%- if", "{%", "%}", Construct::If),
                ("{% else", "{%", "%}", Construct::Else),
                ("{%- else", "{%", "%}", Construct::Else),
                ("{% elif", "{%", "%}", Construct::Else),
                ("{%- elif", "{%", "%}", Construct::Else),
                ("{% for", "{%", "%}", Construct::Loop),
                ("{%- for", "{%", "%}", Construct::Loop),
                ("{% block", "{%", "%}", Construct::Block),
                ("{%- block", "{%", "%}", Construct::Block),
                ("{% raw", "{%", "%}", Construct::Block),
                ("{%- raw", "{%", "%}", Construct::Block),
                ("{% macro", "{%", "%}", Construct::Block),
                ("{%- macro", "{%", "%}", Construct::Block),
                ("{% with", "{%", "%}", Construct::Block),
                ("{%- with", "{%", "%}", Construct::Block),
                ("{% endif", "{%", "%}", Construct::EndIf),
                ("{%- endif", "{%", "%}", Construct::EndIf),
                ("{% endfor", "{%", "%}", Construct::EndLoop),
                ("{%- endfor", "{%", "%}", Construct::EndLoop),
                ("{% endblock", "{%", "%}", Construct::EndBlock),
                ("{%- endblock", "{%", "%}", Construct::EndBlock),
                ("{% endraw", "{%", "%}", Construct::EndBlock),
                ("{%- endraw", "{%", "%}", Construct::EndBlock),
                ("{% endmacro", "{%", "%}", Construct::EndBlock),
                ("{%- endmacro", "{%", "%}", Construct::EndBlock),
                ("{% endwith", "{%", "%}", Construct::EndBlock),
                ("{%- endwith", "{%", "%}", Construct::EndBlock),
                ("{{", "{{", "}}", Construct::Expression),
                ("{%", "{%", "%}", Construct::Statement),
                ("{#", "{#", "#}", Construct::Comment),
            ],
            TemplateLanguage::Liquid => vec![
                ("{% if", "{%", "%}", Construct::If),
                ("{%- if", "{%", "%}", Construct::If),
                ("{% else", "{%", "%}", Construct::Else),
                ("{%- else", "{%", "%}", Construct::Else),
                ("{% elsif", "{%", "%}", Construct::Else),
                ("{%- elsif", "{%", "%}", Construct::Else),
                ("{% for", "{%", "%}", Construct::Loop),
                ("{%- for", "{%", "%}", Construct::Loop),
                ("{% case", "{%", "%}", Construct::Switch),
                ("{%- case", "{%", "%}", Construct::Switch),
                ("{% when", "{%", "%}", Construct::Case),
                ("{%- when", "{%", "%}", Construct::Case),
                ("{% raw", "{%", "%}", Construct::Block),
                ("{%- raw", "{%", "%}", Construct::Block),
                ("{% comment", "{%", "%}", Construct::Block),
                ("{%- comment", "{%", "%}", Construct::Block),
                ("{% capture", "{%", "%}", Construct::Block),
                ("{%- capture", "{%", "%}", Construct::Block),
                ("{% endif", "{%", "%}", Construct::EndIf),
                ("{%- endif", "{%", "%}", Construct::EndIf),
                ("{% endfor", "{%", "%}", Construct::EndLoop),
                ("{%- endfor", "{%", "%}", Construct::EndLoop),
                ("{% endcase", "{%", "%}", Construct::EndSwitch),
                ("{%- endcase", "{%", "%}", Construct::EndSwitch),
                ("{% endraw", "{%", "%}", Construct::EndBlock),
                ("{%- endraw", "{%", "%}", Construct::EndBlock),
                ("{% endcomment", "{%", "%}", Construct::EndBlock),
                ("{%- endcomment", "{%", "%}", Construct::EndBlock),
                ("{% endcapture", "{%", "%}", Construct::EndBlock),
                ("{%- endcapture", "{%", "%}", Construct::EndBlock),
                ("{{", "{{", "}}", Construct::Expression),
                ("{%", "{%", "%}", Construct::Statement),
            ],
            TemplateLanguage::Mustache => vec![
                ("{{#", "{{", "}}", Construct::Block),
                ("{{^", "{{", "}}", Construct::Block),
                ("{{/", "{{", "}}", Construct::EndBlock),
                ("{{{", "{{{", "}}}", Construct::Expression),
                ("{{", "{{", "}}", Construct::Expression),
            ],
            TemplateLanguage::Twig => vec![
                ("{% if", "{%", "%}", Construct::If),
                ("{%- if", "{%", "%}", Construct::If),
                ("{% else", "{%", "%}", Construct::Else),
                ("{%- else", "{%", "%}", Construct::Else),
                ("{% elseif", "{%", "%}", Construct::Else),
                ("{%- elseif", "{%", "%}", Construct::Else),
                ("{% endif", "{%", "%}", Construct::EndIf),
                ("{%- endif", "{%", "%}", Construct::EndIf),
                ("{% for", "{%", "%}", Construct::Loop),
                ("{%- for", "{%", "%}", Construct::Loop),
                ("{% endfor", "{%", "%}", Construct::EndLoop),
                ("{%- endfor", "{%", "%}", Construct::EndLoop),
                ("{% block", "{%", "%}", Construct::Block),
                ("{%- block", "{%", "%}", Construct::Block),
                ("{% endblock", "{%", "%}", Construct::EndBlock),
                ("{%- endblock", "{%", "%}", Construct::EndBlock),
                ("{% macro", "{%", "%}", Construct::Block),
                ("{%- macro", "{%", "%}", Construct::Block),
                ("{% endmacro", "{%", "%}", Construct::EndBlock),
                ("{%- endmacro", "{%", "%}", Construct::EndBlock),
                ("{% raw", "{%", "%}", Construct::Block),
                ("{%- raw", "{%", "%}", Construct::Block),
                ("{% endraw", "{%", "%}", Construct::EndBlock),
                ("{%- endraw", "{%", "%}", Construct::EndBlock),
                ("{% embed", "{%", "%}", Construct::Block),
                ("{%- embed", "{%", "%}", Construct::Block),
                ("{% endembed", "{%", "%}", Construct::EndBlock),
                ("{%- endembed", "{%", "%}", Construct::EndBlock),
                ("{% verbatim", "{%", "%}", Construct::Block),
                ("{%- verbatim", "{%", "%}", Construct::Block),
                ("{% endverbatim", "{%", "%}", Construct::EndBlock),
                ("{%- endverbatim", "{%", "%}", Construct::EndBlock),
                ("{% filter", "{%", "%}", Construct::Block),
                ("{%- filter", "{%", "%}", Construct::Block),
                ("{% endfilter", "{%", "%}", Construct::EndBlock),
                ("{%- endfilter", "{%", "%}", Construct::EndBlock),
                ("{% spaceless", "{%", "%}", Construct::Block),
                ("{%- spaceless", "{%", "%}", Construct::Block),
                ("{% endspaceless", "{%", "%}", Construct::EndBlock),
                ("{%- endspaceless", "{%", "%}", Construct::EndBlock),
                ("{{", "{{", "}}", Construct::Expression),
                ("{%", "{%", "%}", Construct::Statement),
                ("{#", "{#", "#}", Construct::Comment),
            ],
            TemplateLanguage::Eex => vec![
                ("<% if", "<%", "%>", Construct::If),
                ("<%= if", "<%", "%>", Construct::If),
                ("<% unless", "<%", "%>", Construct::If),
                ("<%= unless", "<%", "%>", Construct::If),
                ("<% else", "<%", "%>", Construct::Else),
                ("<% elsif", "<%", "%>", Construct::Else),
                ("<% for", "<%", "%>", Construct::Loop),
                ("<%= for", "<%", "%>", Construct::Loop),
                ("<% case", "<%", "%>", Construct::Block), // hack as we can't detect EndCase
                ("<%= case", "<%", "%>", Construct::Block), // hack as we can't detect EndCase
                ("<% do ", "<%", "%>", Construct::Block),
                ("<%= do ", "<%", "%>", Construct::Block),
                ("<% begin", "<%", "%>", Construct::Block),
                ("<%= begin", "<%", "%>", Construct::Block),
                ("<% rescue", "<%", "%>", Construct::Else),
                ("<%= rescue", "<%", "%>", Construct::Else),
                ("<% ensure", "<%", "%>", Construct::Else),
                ("<%= case", "<%", "%>", Construct::Block),
                ("<% cond", "<%", "%>", Construct::Switch),
                ("<%= cond", "<%", "%>", Construct::Switch),
                ("<% end", "<%", "%>", Construct::EndBlock),
                ("<%=", "<%", "%>", Construct::Expression),
                ("<%==", "<%", "%>", Construct::Expression),
                ("<%", "<%", "%>", Construct::Statement),
            ],
            TemplateLanguage::Erb => vec![
                // TODO: Handle ERB constructs more accurately to allow indentation
                //("<% if", "<%", "%>", Construct::If),
                //("<% else", "<%", "%>", Construct::Else),
                //("<% elsif", "<%", "%>", Construct::Else),
                //("<% end", "<%", "%>", Construct::EndBlock),
                //("<% for", "<%", "%>", Construct::Loop),
                //("<% case", "<%", "%>", Construct::Switch),
                //("<% when", "<%", "%>", Construct::Case),
                //("<% do", "<%", "%>", Construct::Block),
                //("<% begin", "<%", "%>", Construct::Block),
                //("<% rescue", "<%", "%>", Construct::Else),
                //("<% ensure", "<%", "%>", Construct::Else),
                ("<%=", "<%", "%>", Construct::Expression),
                ("<%", "<%", "%>", Construct::Statement),
            ],
            TemplateLanguage::Go => vec![
                ("{{if", "{{", "}}", Construct::If),
                ("{{else", "{{", "}}", Construct::Else),
                ("{{else if", "{{", "}}", Construct::Else),
                ("{{end", "{{", "}}", Construct::EndBlock),
                ("{{range", "{{", "}}", Construct::Loop),
                ("{{with", "{{", "}}", Construct::Block),
                ("{{template", "{{", "}}", Construct::Block),
                ("{{block", "{{", "}}", Construct::Block),
                ("{{define", "{{", "}}", Construct::Block),
                ("{{call", "{{", "}}", Construct::Block),
                ("{{", "{{", "}}", Construct::Expression),
                ("{{/*", "{{/*", "*/}}", Construct::Comment),
            ],
        }
    }
}
