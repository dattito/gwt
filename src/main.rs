use clap::{Parser, Subcommand};

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
    },
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
        } => commands::add_worktree(branch_name, *copy, *verbose),
        Commands::Sync { copy } => commands::sync_worktrees(*copy),
        Commands::Clone { repo } => commands::clone_repo(repo),
        Commands::Init {} => commands::init_gwtconfig(),
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
