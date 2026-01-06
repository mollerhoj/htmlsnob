use magnus::{function, prelude::*, Error, RHash, Ruby, Symbol};

fn run_simple(file_paths: Vec<String>, config: Option<String>) -> (i32, String) {
    htmlsnob_cli::run(htmlsnob_cli::SimpleArgs {
        paths: file_paths,
        autofix: false,
        config,
        ignore: Vec::new(),
        registry: htmlsnob_rules::registry(),
    })
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HtmlsnobRuby")?;
    module.define_singleton_method("run_simple", function!(run_simple, 2))?;
    Ok(())
}
