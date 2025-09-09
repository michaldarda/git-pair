# git-pair

A Git extension for managing pair programming sessions. Easily configure Git to commit with multiple authors when pair programming.

## Overview

`git-pair` helps teams that practice pair programming by simplifying the process of attributing commits to multiple authors. It provides a clean interface to manage co-authors and automatically formats commit messages with proper Co-authored-by trailers.

## Features

- 🤝 **Easy pair setup**: Quickly initialize and configure pair programming sessions
- 👥 **Multiple authors**: Add multiple co-authors to your commits
- 🧹 **Clean state management**: Clear pair configuration when switching between solo and pair work
- 📝 **Proper attribution**: Follows Git's standard Co-authored-by trailer format
- ⚡ **Fast and lightweight**: Written in Rust for optimal performance

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

Shows the current working directory and initializes pair programming mode.

### Add Co-authors

```bash
git pair add "Jane Doe" jane.doe@company.com
git pair add "John Smith" john.smith@company.com
```

Adds co-authors to the current pair programming session. You can add multiple co-authors.

### Clear Pair Configuration

```bash
git pair clear
```

Removes all co-authors and exits pair programming mode, returning to solo development.

### View Current Pair Status

```bash
git pair status
```

Displays the currently configured co-authors and pair programming status.

## How It Works

When you add co-authors using `git pair add`, the tool configures Git to automatically include Co-authored-by trailers in your commit messages. This follows GitHub's standard for attributing commits to multiple authors.

Example commit message with pair programming:
```
Implement user authentication feature

Co-authored-by: Jane Doe <jane.doe@company.com>
Co-authored-by: John Smith <john.smith@company.com>
```

## Configuration

`git-pair` stores its configuration in your Git repository's local configuration (`.git/config`). This means:

- Configuration is per-repository
- No global state pollution
- Easy to switch between different pairing configurations for different projects

## Commands Reference

| Command | Description |
|---------|-------------|
| `git pair init` | Initialize pair programming in current repository |
| `git pair add <name> <email>` | Add a co-author to the current session |
| `git pair clear` | Remove all co-authors and exit pair mode |
| `git pair status` | Show current pair configuration |
| `git pair list` | List all previously used co-authors |

## Examples

### Starting a Pair Session

```bash
# Initialize pair programming
git pair init

# Add your pair programming partner
git pair add "Alice Johnson" alice@company.com

# Make commits as usual - they'll automatically include co-author attribution
git commit -m "Add new feature"
```

### Working with Multiple People

```bash
# Add multiple co-authors for mob programming
git pair add "Bob Wilson" bob@company.com
git pair add "Carol Davis" carol@company.com
git pair add "Dave Miller" dave@company.com

# Check current status
git pair status
```

### Switching Back to Solo Work

```bash
# Clear all co-authors when pair session ends
git pair clear
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
- **Fast**: Written in Rust for optimal performance
- **Standard**: Uses Git's built-in Co-authored-by trailer format
- **Local**: Repository-specific configuration without global pollution

---

Made with ❤️ for pair programmers everywhere.
