# GitHub Copilot Instructions for git-pair

## Project Overview

git-pair is a Git extension for managing pair programming sessions, written in Rust. It provides per-branch co-author configuration and a global roster for quick access to frequent collaborators.

## Core Architecture

### Key Components
- **Per-branch configuration**: Each git branch maintains its own co-author list in `.git/git-pair/config-{branch_name}`
- **Global roster**: Shared co-author aliases stored in `~/.config/git-pair/roster`
- **Git hook integration**: Automatic `prepare-commit-msg` hook that adds Co-authored-by trailers
- **CLI interface**: Simple commands for managing pair programming sessions

### File Structure
```
src/
├── main.rs          # CLI entry point and command parsing
└── lib.rs           # Core functionality and business logic

.git/git-pair/
├── config-main      # Co-authors for main branch
├── config-feature   # Co-authors for feature branch
└── ...

~/.config/git-pair/
└── roster           # Global alias -> name/email mapping
```

## Code Patterns & Conventions

### Error Handling
- Use `Result<T, String>` for functions that can fail
- Provide descriptive error messages that help users understand issues
- Prefer `map_err()` to convert system errors to user-friendly strings

```rust
fs::write(&config_file, content)
    .map_err(|e| format!("Error writing config file: {}", e))?;
```

### File Operations
- Always use absolute paths when possible
- Create directories with `fs::create_dir_all()` before writing files
- Use proper error handling for all file operations

### Git Integration
- Use `std::process::Command` to execute git commands
- Always check command exit status with `output.status.success()`
- Parse git output carefully, handling empty results

### Branch Name Sanitization
- Replace problematic characters in branch names for filenames:
  ```rust
  let safe_name = branch_name.replace(['/', '\\', ':'], "_");
  ```

### Testing Patterns
- Use temporary directories for isolated tests
- Set up proper git configuration in test repos
- Use environment variables for test isolation (e.g., `GIT_PAIR_ROSTER_FILE`)
- Include both positive and negative test cases

## CLI Design Principles

### Command Structure
- Keep commands simple and intuitive
- Support both direct addition and global roster workflows
- Provide clear error messages and helpful suggestions
- Follow git command conventions where possible

### User Experience
- Branch-specific configuration should be transparent to users
- Global roster should work across all repositories
- Commands should be fast and provide immediate feedback
- Help text should include practical examples

## Development Guidelines

### When Adding New Features
1. Consider both per-branch and global roster implications
2. Add corresponding unit tests
3. Update integration tests if needed
4. Update CLI help text and README
5. Ensure proper error handling and user feedback

### Code Style
- Follow Rust standard formatting (rustfmt)
- Use clippy suggestions to improve code quality
- Prefer explicit error handling over unwrap()
- Use descriptive variable and function names
- **Avoid using external dependencies** - keep the project lean and dependency-free
- **Always run `./check.sh`** to test if everything works as expected before suggesting changes

### Testing Strategy
- Unit tests for core functionality in `src/lib.rs`
- Integration tests via `integration_test.sh` for end-to-end CLI testing
- Test error conditions and edge cases
- Ensure tests are isolated and can run in parallel

## Common Operations

### Adding Co-authors
```rust
// Direct addition
add_coauthor("John", "Doe", "john@example.com")?;

// From global roster
add_coauthor_from_global("alias")?;

// To global roster
add_global_coauthor("alias", "John Doe", "john@example.com")?;
```

### File Path Handling
```rust
// Get branch-specific config file
let config_file = get_branch_config_file()?;

// Get global roster file (with environment override)
let roster_file = env::var("GIT_PAIR_ROSTER_FILE")
    .unwrap_or_else(|_| format!("{}/.config/git-pair/roster", env::var("HOME").unwrap()));
```

### Git Hook Management
```rust
// Install hook
install_git_hook()?;

// Remove hook (when clearing co-authors)
remove_git_hook()?;
```

## Security & Privacy Considerations

- Never store sensitive information in configuration files
- Respect user's home directory structure
- Use environment variables for configuration overrides
- Ensure proper file permissions on created files

## Performance Guidelines

- Minimize file I/O operations
- Cache git repository information when possible
- Use efficient string operations
- Avoid unnecessary git command executions
- **Keep dependencies minimal** - prefer standard library solutions over external crates

## Development Workflow

- **Always run `./check.sh`** before committing changes to ensure all checks pass
- Use `./fix.sh` to apply automatic formatting and clippy fixes
- Run `./integration_test.sh` for end-to-end testing
- The check script validates: formatting, linting, unit tests, release build, and integration tests

## Compatibility Notes

- Support Unix-like systems (Linux, macOS)
- Windows compatibility for file operations
- Work with standard git installations
- Handle different git repository structures

## Future Considerations

When suggesting improvements or new features, consider:
- Maintaining backward compatibility
- Keeping the CLI simple and focused
- Supporting team workflows and collaboration patterns
- Integration with other development tools
- Performance impact on large repositories

## Error Message Guidelines

- Be specific about what went wrong
- Suggest corrective actions when possible
- Include relevant context (branch names, file paths)
- Use consistent terminology throughout the application

Example good error message:
```
Error: git-pair not initialized for branch 'feature/auth'. Please run 'git-pair init' first.
```

## Testing Utilities

The project includes helper functions for testing:
- `TempDir` for isolated test environments
- `create_temp_file()` for temporary file creation
- `setup_test_repo()` for git repository setup
- Environment variable isolation for global roster tests

Use these patterns when adding new tests to ensure consistency and reliability.
