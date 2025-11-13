use clap::Parser;

/// A CLI tool to help create better commit messages
#[derive(Parser, Debug)]
#[command(name = "commit-wizard")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
    }

    println!("Welcome to Commit Wizard!");
    println!("This tool helps you create better commit messages.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        // Basic test to ensure the CLI structure is valid
        let cli = Cli { verbose: false };
        assert!(!cli.verbose);
    }
}
