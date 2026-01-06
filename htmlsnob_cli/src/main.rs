use clap::Parser;
use htmlsnob_cli::run;
use htmlsnob_cli::SimpleArgs;
use std::process;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input paths
    #[arg(default_value = "**/*.html")]
    pub paths: Vec<String>,

    /// Correct the issues inline when possible
    #[arg(short, long)]
    pub autofix: bool,

    /// Config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Files or patterns to ignore (can be specified multiple times)
    #[arg(short, long)]
    pub ignore: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let registry = htmlsnob_rules::registry();

    let (status, message) = run(SimpleArgs {
        paths: args.paths,
        autofix: args.autofix,
        config: args.config,
        ignore: args.ignore,
        registry,
    });

    print!("{}", message);
    process::exit(status);
}
