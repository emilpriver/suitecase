mod test;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, ValueEnum, Default, PartialEq)]
enum OutputMode {
    #[default]
    Tui,
    Github,
}

#[derive(Parser)]
#[command(
    name = "suitecase",
    about = "Run tests with formatted suitecase output"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run cargo test with formatted suitecase output
    Test {
        /// Output format: tui (default), github (GitHub Actions log grouping with collapsible test output)
        #[arg(long, default_value = "tui")]
        output: OutputMode,

        /// Arguments passed directly to cargo test
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { args, output } => {
            test::run(args, output);
        }
    }
}
