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
    about = "Run tests with formatted suitecase output",
    version = env!("CARGO_PKG_VERSION"),
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

        /// Build and test with all workspace members
        #[arg(long)]
        workspace: bool,

        /// Build artifacts in release mode with optimizations
        #[arg(long)]
        release: bool,

        /// Arguments passed directly to cargo test
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { args, output, workspace, release } => {
            test::run(args, output, workspace, release);
        }
    }
}
