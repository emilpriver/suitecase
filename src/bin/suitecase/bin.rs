mod test;

use clap::{Parser, Subcommand};

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
        /// Arguments passed directly to cargo test
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { args } => {
            test::run(args);
        }
    }
}
