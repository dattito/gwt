# gwt - Git Worktree Manager

`gwt` is a command-line tool designed to streamline the management of Git worktrees, especially for developers who frequently switch between branches or work on multiple features simultaneously. It extends Git's built-in worktree functionality with features for automatic file synchronization and simplified repository cloning.

## Features

- **Efficient Worktree Creation:** Quickly create new worktrees with intelligent handling of common development files.
- **Symbolic Linking by Default:** Automatically creates symbolic links for specified files and directories, ensuring a single source of truth across multiple worktrees and saving disk space.
- **Cross-Worktree Synchronization:** Keep important files (like configuration or build outputs) synchronized across all your worktrees.
- **Streamlined Cloning:** Clone repositories and set up a worktree-ready environment with a single command.
- **Interactive Configuration:** Easily generate a `.gwtconfig` file based on your existing `.gitignore`.

## Installation

To install `gwt`, make sure you have Rust and Cargo installed, then run:

```bash
cargo install --git https://github.com/dattito/gwt
```

## Usage

`gwt` provides several subcommands to manage your worktrees:

### `gwt add <branch-name>`

Creates a new Git worktree for the specified branch. By default, `gwt` will attempt to create symbolic links for files and directories listed in your `.gwtconfig` file. This ensures that changes to these files are instantly reflected across all linked worktrees.

**Arguments:**

- `<branch-name>`: The name of the branch for which to create the worktree. If the branch doesn't exist, it will be created.

**Options:**

- `--copy` / `-c`: Forces `gwt` to copy files instead of creating symbolic links. Use this if you need independent copies of the files in the new worktree.
- `--verbose` / `-v`: Enables verbose output.

**Example:**

```bash
gwt add feature/my-new-feature
# This will create a worktree for 'feature/my-new-feature' and symlink files from .gwtconfig

gwt add bugfix/issue-123 --copy
# This will create a worktree for 'bugfix/issue-123' and copy files from .gwtconfig
```

### `gwt sync`

Synchronizes files and directories listed in your `.gwtconfig` across all existing Git worktrees in the current repository. `gwt` will find the most recently modified version of each file and update all other worktrees accordingly. By default, it attempts to create symbolic links; if linking fails (e.g., due to cross-device issues), it falls back to copying.

**Options:**

- `--copy` / `-c`: Forces `gwt` to copy files instead of attempting to create symbolic links.

**Example:**

```bash
gwt sync
# Synchronizes files across all worktrees, preferring symlinks

gwt sync --copy
# Synchronizes files across all worktrees by copying them
```

### `gwt clone <repo>`

Clones a Git repository and sets up a `gwt`-friendly worktree structure. This command creates a bare repository in a hidden `.bare` directory and then initializes the default branch as the first worktree.

**Arguments:**

- `<repo>`: The repository to clone. Can be a full URL or a `owner/repo` string (requires GitHub CLI `gh` to be installed and configured).

**Example:**

```bash
gwt clone octocat/Spoon-Knife
# Clones the Spoon-Knife repository, sets up .bare, and creates a 'main' worktree

gwt clone https://github.com/rust-lang/rust
# Clones the Rust repository and sets up its default branch as the initial worktree
```

### `gwt init`

Interactively helps you create or update your `.gwtconfig` file. It reads your `.gitignore` and suggests patterns (files or directories) that you might want to include in your `.gwtconfig` for automatic linking or copying.

**Example:**

```bash
gwt init
# Starts an interactive session to build your .gwtconfig
```

## `.gwtconfig` File

The `.gwtconfig` file is a plain text file located in the root of your Git repository. Each line in this file specifies a path (relative to the repository root) to a file or directory that `gwt` should manage (link or copy) across your worktrees.

**Example `.gwtconfig`:**

```
.env
node_modules/
target/debug/my_app
```

Lines starting with `#` are treated as comments and ignored. Empty lines are also ignored.
