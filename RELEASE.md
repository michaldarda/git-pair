# Release Process

This document describes how to create a new release of git-pair.

## Automated Release Process

The project uses GitHub Actions for automated testing and releases.

### On Every Push/PR

The CI workflow (`.github/workflows/ci.yml`) automatically:
- ✅ Runs tests on Rust stable, beta, and nightly
- ✅ Runs clippy linting
- ✅ Checks code formatting with rustfmt
- ✅ Builds the project in debug and release mode

### On Version Tag Push

The release workflow (`.github/workflows/release.yml`) automatically:
- ✅ Runs full test suite
- ✅ Builds binaries for multiple platforms:
  - Linux (x86_64-unknown-linux-gnu)
  - Linux musl (x86_64-unknown-linux-musl) 
  - macOS Intel (x86_64-apple-darwin)
  - macOS Apple Silicon (aarch64-apple-darwin)
  - Windows (x86_64-pc-windows-msvc)
- ✅ Creates GitHub release with binaries
- ✅ Publishes to crates.io (if CARGO_REGISTRY_TOKEN is set)

## Creating a Release

### 1. Prepare the Release

1. **Update CHANGELOG.md**:
   ```bash
   # Move items from [Unreleased] to new version section
   # Add new [Unreleased] section at top
   ```

2. **Update version in Cargo.toml**:
   ```bash
   # Bump version following semantic versioning
   # 0.1.0 -> 0.1.1 (patch)
   # 0.1.0 -> 0.2.0 (minor) 
   # 0.1.0 -> 1.0.0 (major)
   ```

3. **Test locally**:
   ```bash
   cargo test
   cargo build --release
   ./target/release/git-pair --version
   ```

4. **Commit changes**:
   ```bash
   git add CHANGELOG.md Cargo.toml Cargo.lock
   git commit -m "Release v0.1.1"
   git push origin master
   ```

### 2. Create and Push Tag

```bash
# Create tag matching version in Cargo.toml
git tag v0.1.1

# Push tag to trigger release workflow
git push origin v0.1.1
```

### 3. Monitor Release

1. **Check GitHub Actions**: Visit the Actions tab to monitor the release workflow
2. **Verify Release**: Check the Releases page for the new release with binaries
3. **Check crates.io**: Verify the package was published (if token is configured)

## Setting Up Secrets

For full automation, configure these secrets in your GitHub repository:

### Required Secrets

- **GITHUB_TOKEN**: Automatically provided by GitHub (no setup needed)

### Optional Secrets

- **CARGO_REGISTRY_TOKEN**: For publishing to crates.io
  1. Get token from https://crates.io/me
  2. Add as repository secret in GitHub Settings > Secrets

## Manual Release (Backup)

If automated release fails, you can create releases manually:

### Build Binaries
```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS (if on macOS)
cargo build --release --target x86_64-apple-darwin

# Windows (if on Windows)
cargo build --release --target x86_64-pc-windows-msvc
```

### Create GitHub Release
1. Go to GitHub Releases page
2. Click "Create a new release"
3. Choose tag version (or create new tag)
4. Add release notes from CHANGELOG.md
5. Upload binary files
6. Publish release

### Publish to crates.io
```bash
cargo publish
```

## Version Strategy

git-pair follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: Backward-compatible functionality additions  
- **PATCH** version: Backward-compatible bug fixes

### Examples:
- `0.1.0` → `0.1.1`: Bug fixes
- `0.1.0` → `0.2.0`: New features (global roster)
- `0.1.0` → `1.0.0`: Stable API, major changes
