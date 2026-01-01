<img src="htmlsnob.png" align="right"
     alt="The htmlsnob himself" width="128" height="128">

_"A gentleman writes their HTML by hand"_
# HTMLsnob

Validates and formats HTML and template languages (Handlebars, Mustache, Jinja2, Twig, Eex, Erb, Blade, Ejs, Go, etc.).
The highly configurable architecture allows you to enforce your exact coding standards. Works directly in your editor via LSP or as part of your CI/CD pipeline.

# Key Features

- Multi-language Support: Works with Handlebars, Mustache, Jinja2, Twig, Eex, Erb, Blade, Ejs, Go, and more.

- Deeply Configurable: Enforce your exact team standards, from attribute ordering to accessibility requirements.

- Integrated Workflow: Use it as a CLI tool or a Language Server (LSP) for real-time feedback.

- Autofix Support: Automatically correct formatting and minor syntax issues on save.

# Validation examples

```
<html> -- Missing required attributes: lang
  <body>
    <p ="my-class">Hello World</p> -- Attribute name missing, expected a name before the = sign

    <p Class="my-class">Hello World</p> -- Attribute name `Class` should be in kebab-case, change to `class`

    <canvas xyz></canvas> -- `xyz` not allowed, must be a global attribute or: `height, width`

    <p class="myClass">Hello World</p> -- Attribute value `myClass` should be in kebab-case, change to `my-class`

    <p class='my-class'>Hello World</p> -- Attribute value must be quoted with ""

    <button type="clickable"></button> -- Attribute value `clickable` must be one of `submit, reset, button`

    <p id="my-id" class="my-class">Hello World</p> -- Attribute `class` must be before `id`

    <input checked="true" /> -- Boolean Attribute must have no value `checked`

    <p class="my-class" class="another-class">Hello World</p> -- Attribute "class" appears more than once

    <p class="flex-row flex">Hello World</p> -- `flex` must be before `flex-row`

    <p id="SomeId">Hello World</p> -- Attribute value `SomeId` should be in kebab-case, change to `some-id`

    <p id="a">Hello World</p><span id="a"></span> -- id value `a` is used more than once

    <ul>Hello World<li></li></ul> -- `ul` tags must not contain text content

    <p>  </p> -- `p` tags must contain text

    <homemade> </homemade> -- Tag `homemade` is not allowed

    <1-p></1-p> -- Tag name `1-p` must match the regexp `^[A-Za-z][A-Za-z0-9.-]*$`

    <P></P> -- Tag name "P" should be in lowercase, change to "p"
  </body>
</html>
```

# Installation

## VSCode

Add the [HTMLsnob extension](https://marketplace.visualstudio.com/items?itemName=mollerhoj.htmlsnob-vscode-extension)

## CLI

`cargo install htmlsnob`

```
htmlsnob_cli --help
Check HTML files for issues

Usage: htmlsnob_cli [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Input paths [default: **/*.html]

Options:
  -a, --autofix          Correct the issues inline when possible
  -c, --config <CONFIG>  Config file
  -i, --ignore <IGNORE>  Files or patterns to ignore (can be specified multiple times)
  -h, --help             Print help
  -V, --version          Print version
```

## Neovim

Download the binary as shown in the CLI section above, and use it:
```
local configs = require('lspconfig.configs')

if not configs.htmlsnob_lsp then
  configs.htmlsnob_lsp = {
    default_config = {
      cmd = { '<PATH TO BINARY>/htmlsnob_lsp' },
      filetypes = {
        'html',
        'handlebars',
        -- add more filetypes as needed
      },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
      settings = {},
      on_attach = function(client, bufnr)
        vim.api.nvim_create_autocmd("BufWritePre", {
          buffer = bufnr,
          callback = function()
            vim.lsp.buf.format({ async = false })
          end,
        })
      end,
    },
  }
end

vim.filetype.add({
  pattern = {
    ["*.gotmpl"] = "gotmpl",
    ["*.tmpl"] = "gotmpl",
    ["*.gohtml"] = "gotmpl",
    ["*.ejs"] = "ejs",
    ["*.liquid"] = "liquid",
    ["*.mustache"] = "mustache",
  },
})

lspconfig.htmlsnob_lsp.setup {}
```
If you wanna make a plugin for this, please do! I would be happy to include it in this repo.

# Recommended Usage

While the default configuration enforces parts of the html specifications, HTMLsnob is designed to help you enforce your own much stricter coding standards.

- Enforce that all links are relative using a regular expression
- Enforce that all text content is translated using a specific function call
- Enforce that all `img` tags have `alt` attributes
- Enforce that no inline styles are used
- Enforce that only a specific set of HTML tags are used
- Enforcing accessibility standards
- Accepting only email-safe HTML
- Accept only a subset of HTML for supported by cross-platform HTML-to-native frameworks

While many validations can be implemented by configuring the rules shipped with HTMLsnob, you can also implement your own rules in a separate crate and use HTMLsnob to validate them. See the "Extending with New Rules" section below.

# Configuration

## Rule configuration

Rules a configured via a toml config file. See `default_config.toml` for the default rules. This should give you an idea of how to configure your own rules.

Each rule has a `kind` which links it to a specific rule implementation. For example, this rule enforces that attribute names are in kebab-case:
```
[[rules]]
name = "attribute_name_casing_style"  # Unique name for the rule used in error messages. Defaults to the kind if not specified
kind = "attribute_name_casing_style"  # Links to the rule implementation
severity = "warning"                  # Usually your editor will show different severities differently
autofix = true                        # Whether to autofix issues found by this rule
style = "kebab_case"                  # The desired casing style, can be "kebab_case", "snake_case", "camel_case", or "pascal_case"
```

## Ignoring lines

If you want HTMLsnob to ignore parts of your html files, add `ignore below` and `ignore above` comments to your file:
```
<!-- ignore below -->
<p>Any issues here will be ignored.</p>
<!-- ignore above -->
<p>Any issues here will not be ignored.</p>
```

### Expansions
The config file supports expansions, which allows you to define variables that can be used throughout the config file. This is useful for defining common values that are used in multiple places.

The special key `expansions` is used to define the variables, and these can be used as both keys and values in the rules:
```toml
[expansions]
RAW_TEXT = ["script", "style"]

[rules.some_rule]
key = ["pre", RAW_TEXT]
RAW_TEXT = ["values"]
```

This will expand to:

```toml
[rules.some_rule]
key = ["pre", "script", "style"]
script = ["values"]
style = ["values"]
```

# Supported Template languages

- [Eex](https://hexdocs.pm/eex/EEx.html),
- [Handlebars](https://handlebarsjs.com/),
- [Jinja2](https://jinja.palletsprojects.com/),
- [Liquid](https://shopify.github.io/liquid/),
- [Mustache](https://mustache.github.io/),

#### Template languages with limited support

- [Erb](https://guides.rubyonrails.org/layouts_and_rendering.html#erb-templates),
- [Go](https://golang.org/pkg/html/template/),
- [Twig](https://twig.symfony.com/),

# Supported Rules

See `htmlsnob_rules/src/lib.rs`

# Architecture

HTMLsnob uses a high-efficiency, single-pass engine for parsing, validation, and auto-formatting. By processing tokens sequentially, the engine ensures that auto-formatting in the early parts of a document is reflected in the validation of later sections.

Validation rules function as hooks within the parsing lifecycle, allowing them to flag issues and collect warnings without requiring a second pass of the document. 

```
htmlsnob: The core engine handling parsing and formatting - independent of specific validations and autofixes
htmlsnob_rules: A package with a standard set of validation rules 
htmlsnob_cli: A command line interface that uses the htmlsnob engine
htmlsnob_lsp: A Language Server that the uses the htmlsnob engine
htmlsnob_vscode_extension: A VS Code extension that uses the htmlsnob engine
```

# Extending with New Rules

HTMLsnob is designed to be easily extensible with new rules.

To implement a new rule, implement the `RuleTrait`. To use the new rule, add it to the Registry. A minimal example is shown below.

```
use htmlsnob_engine::{
    ast::OpenTag, config::Config, lint, parser::ParseState, registry::Registry,
    rule_trait::RuleTrait, warning::Warning, WarningSeverity,
};
use serde::Deserialize;

/// Enforces that an elements specfied attribute does not have any of the specified values.
#[derive(Debug, Clone, Deserialize)]
pub struct MinimalRule {
    #[serde(default)]
    name: String,
    kind: String,
}

impl RuleTrait for MinimalRule {
    // We must implement a method that takes a Node from the DOM, and returns one or more Warnings.
    // In this case, we will implement apply_open_tag, which is called on each opening tag.
    // Notice the OpenTag is mutable. This allows us to autofix any issues if desired.
    // See the RuleTrait definition for other methods that can be implemented.
    fn apply_open_tag(&self, open_tag: &mut OpenTag, _parse_state: &ParseState) -> Option<Warning> {
        // For demonstration purposes, let's say we want to warn on every <p> tag.
        if open_tag.name == "p" {
            return Some(Warning::from_area(
                &self.name,
                &self.kind,
                // The `area` of the open tag is used to point to the location in the source code.
                open_tag.area.clone(),
                "This is the error message shown to the user",
                WarningSeverity::WARNING, // You can set the severity as desired.
            ));
        }

        None
    }
}

pub fn main() {
    // Register the new rule in the registry.
    let registry = Registry::new().register_rule::<MinimalRule>("minimal_rule");
    // Create a config that uses the new rule.
    let config_str = r#"
        [[rules]]
        kind = "minimal_rule"
    "#;
    // Parse the config.
    let mut config = Config::from_toml(config_str, &registry);
    // Lint some HTML.
    let (ast, warnings) = lint("<p></p>", &mut config);
    println!("AST: {:?}, Warnings: {:?}", ast, warnings);
}
```

## Contributing

Contributions are very welcome! Feature requests belong in the discussion section of the repository, issues are for bug reports and accepted feature requests.

## Licence

MIT License

Copyright (c) 2025 Jens Dahl Møllerhøj

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
