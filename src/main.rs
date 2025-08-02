use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod config;
mod direnv_utils;
mod file_ops;
mod git_utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a new git worktree
    Add {
        /// The name of the branch to create a worktree for
        branch_name: String,

        /// Copy files instead of creating symbolic links (linking is default)
        #[arg(short, long, default_value_t = false)]
        copy: bool,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Run git pull internally before creating worktree
        #[arg(short, long)]
        pull: bool,
    },
    /// Remove a new git worktree and the local branch
    Remove { branch_name: String },
    /// Sync files between worktrees
    Sync {
        /// Copy files instead of creating symbolic links (linking is default)
        #[arg(short, long, default_value_t = false)]
        copy: bool,
    },
    /// Clones a repository and sets it up for gwt worktree usage
    Clone {
        /// Repository to clone (e.g., 'owner/repo' or a full URL)
        repo: String,
    },
    /// Initializes a .gwtconfig file based on .gitignore
    Init {},
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Add {
            branch_name,
            copy,
            verbose,
            pull,
        } => commands::add_worktree(branch_name, *copy, *verbose, *pull),
        Commands::Sync { copy } => commands::sync_worktrees(*copy),
        Commands::Clone { repo } => commands::clone_repo(repo),
        Commands::Init {} => commands::init_gwtconfig(),
        Commands::Remove { branch_name } => commands::remove_worktree(branch_name),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "Error:".red().bold());
        std::process::exit(1);
    }
}
