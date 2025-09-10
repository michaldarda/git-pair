# git-pair

A Git extension for managing pair programming sessions. Easily configure Git to commit with multiple authors when pair programming.

## Overview

`git-pair` helps teams that practice pair programming by simplifying the process of attributing commits to multiple authors. It provides a clean interface to manage co-authors and automatically formats commit messages with proper Co-authored-by trailers.

## Features

- 🤝 **Easy pair setup**: Quickly initialize and configure pair programming sessions
- 🌿 **Per-branch co-authors**: Different co-authors for different branches - perfect for feature teams
- 👥 **Multiple authors**: Add multiple co-authors to your commits
- 🔄 **Automatic switching**: Co-authors change automatically when you switch branches
- ⚡ **Global roster**: Save frequently used co-authors with aliases for quick access across all repositories
- 🧹 **Clean state management**: Clear pair configuration per branch when switching between solo and pair work
- 📝 **Proper attribution**: Follows Git's standard Co-authored-by trailer format
- ⚡ **Fast and lightweight**: Written in Rust for optimal performance
- 🎯 **Branch isolation**: Complete separation of co-author configuration between branches

## Installation

### From Source

```bash
git clone https://github.com/michaldarda/git-pair.git
cd git-pair
cargo build --release
cp target/release/git-pair /usr/local/bin/
```

### Using Cargo

```bash
cargo install git-pair
```

## Usage

### Initialize Pair Programming Session

```bash
git pair init
```

Initializes pair programming mode for the current branch. Each branch maintains its own co-author configuration, allowing different teams to work on different features simultaneously.

### Add Co-authors

```bash
# Add co-authors directly
git pair add "Jane Doe" jane.doe@company.com
git pair add "John Smith" john.smith@company.com

# Or add from global roster using aliases
git pair add jane    # Adds Jane Doe from global roster
git pair add john    # Adds John Smith from global roster
```

Adds co-authors to the current branch's pair programming session. Co-authors are branch-specific, so switching branches will use different co-author configurations.

### Clear Pair Configuration

```bash
git pair clear
```

Removes all co-authors from the current branch and exits pair programming mode for this branch, returning to solo development. Other branches maintain their own co-author configurations.

### View Current Pair Status

```bash
git pair status
```

Displays the currently configured co-authors and pair programming status.

### Help and Version

```bash
git pair --help     # Show comprehensive help
git pair --version  # Show version information
```

## How It Works

`git-pair` uses **per-branch configuration** to manage co-authors. When you add co-authors using `git pair add`, the tool creates a branch-specific configuration file and installs a Git hook that automatically includes Co-authored-by trailers in your commit messages.

### Per-Branch Configuration

Each branch gets its own co-author configuration:
- `main` branch → `.git/git-pair/config-main`
- `feature/auth` branch → `.git/git-pair/config-feature_auth`
- `bugfix/login` branch → `.git/git-pair/config-bugfix_login`

When you switch branches, the Git hook automatically reads from the correct configuration file, ensuring the right co-authors are added to commits.

### Automatic Co-author Attribution

The Git hook runs on every commit and automatically appends co-authors to your commit messages. This follows GitHub's standard for attributing commits to multiple authors.

Example commit message with pair programming:
```
Implement user authentication feature

Co-authored-by: Jane Doe <jane.doe@company.com>
Co-authored-by: John Smith <john.smith@company.com>
```

## Configuration

`git-pair` stores its configuration in branch-specific files within `.git/git-pair/` directory. This means:

- **Per-branch configuration**: Each branch has its own co-authors
- **Branch isolation**: Switching branches automatically uses the correct co-authors  
- **No global state pollution**: Configuration is repository-local
- **Easy branch management**: Different teams can work on different branches with their own pair configurations
- **Automatic cleanup**: Deleting a branch doesn't affect other branches' configurations

Example configuration structure:
```
.git/git-pair/
├── config-main                    # Co-authors for main branch
├── config-feature_auth            # Co-authors for feature/auth branch  
└── config-bugfix_login            # Co-authors for bugfix/login branch

~/.config/git-pair/
└── roster                         # Global roster of saved co-authors
```

## Per-Branch Benefits

The per-branch co-author system enables powerful workflows:

- **Feature Teams**: Different features can have different team members without conflicts
- **Parallel Development**: Multiple pairs can work simultaneously on different branches
- **Context Switching**: Switch branches and automatically get the right co-authors
- **Team Flexibility**: Core team on main, specialists on feature branches
- **Clean History**: Each branch's commits reflect the actual contributors to that work

### Use Cases

- **Large Teams**: Different squads working on different features
- **Open Source**: Maintainers on main, contributors on feature branches  
- **Client Work**: Different client teams on different feature branches
- **Skill-based Pairing**: Frontend devs on UI branches, backend devs on API branches

## Commands Reference

| Command | Description |
|---------|-------------|
| `git pair init` | Initialize pair programming for current branch |
| `git pair add <name> <surname> <email>` | Add a co-author to the current branch |
| `git pair add <alias>` | Add co-author from global roster using alias |
| `git pair add --global <alias> <name> <email>` | Add a co-author to global roster |
| `git pair clear` | Remove all co-authors from current branch |
| `git pair status` | Show current branch's pair configuration |
| `git pair list --global` | Show global roster of saved co-authors |
| `git pair --version, -V` | Show version information |
| `git pair --help, -h` | Show help information |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GIT_PAIR_ROSTER_FILE` | Override global roster file location | `~/.config/git-pair/roster` |

Example:
```bash
# Use custom roster file for testing or different environments
GIT_PAIR_ROSTER_FILE=/tmp/test-roster git pair add --global test "Test User" test@example.com
```

## Examples

### Basic Pair Programming

```bash
# Initialize pair programming for current branch
git pair init

# Add your pair programming partner
git pair add "Alice Johnson" alice@company.com

# Make commits as usual - they'll automatically include co-author attribution
git commit -m "Add new feature"
```

### Using Global Roster

```bash
# Set up your global roster once
git pair add --global alice "Alice Johnson" alice@company.com
git pair add --global bob "Bob Wilson" bob@company.com
git pair add --global sarah "Sarah Chen" sarah@company.com

# View your roster
git pair list --global
# Output:
# Global roster:
#   alice -> Alice Johnson <alice@company.com>
#   bob -> Bob Wilson <bob@company.com>
#   sarah -> Sarah Chen <sarah@company.com>

# Quick setup in any repository
git pair init
git pair add alice bob    # Adds both Alice and Bob quickly!

# Make commits - co-authors automatically included
git commit -m "Implement new feature"
```

### Per-Branch Team Configuration

```bash
# On main branch - set up core team using global roster
git checkout main
git pair init
git pair add alice bob    # Quick add from global roster

# Switch to feature branch - set up feature team  
git checkout -b feature/authentication
git pair init
git pair add sarah        # Add Sarah from global roster
git pair add "Carol Davis" carol@company.com  # Add directly

# Commits on feature branch include Sarah and Carol
git commit -m "Implement login system"

# Switch back to main - automatically uses Alice and Bob
git checkout main  
git commit -m "Update documentation"

# Each branch maintains its own co-author configuration!
```

### Working with Multiple People (Mob Programming)

```bash
# Add multiple co-authors for mob programming on current branch
git pair add bob carol dave    # Mix of roster aliases and direct names
git pair add "Eve Foster" eve@company.com

# Check current branch's status
git pair status
```

### Switching Back to Solo Work

```bash
# Clear co-authors for current branch only
git pair clear

# Other branches keep their co-author configurations
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/michaldarda/git-pair.git
cd git-pair
cargo build
cargo test
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Tools

- [git-duet](https://github.com/git-duet/git-duet) - Similar tool written in Go
- [git-together](https://github.com/kejadlen/git-together) - Another pair programming Git extension

## Why Another Pair Programming Tool?

While there are existing solutions, `git-pair` aims to be:

- **Simple**: Minimal commands, maximum productivity
- **Per-branch**: Unique branch-specific co-author configuration - perfect for teams working on multiple features
- **Global roster**: Save frequent collaborators with aliases for lightning-fast setup
- **Fast**: Written in Rust for optimal performance
- **Standard**: Uses Git's built-in Co-authored-by trailer format
- **Local**: Repository-specific configuration without global pollution
- **Isolated**: Complete separation between branches allows different team configurations

---

Made with ❤️ for pair programmers everywhere.
