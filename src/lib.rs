use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_git_pair_dir() -> Result<PathBuf, String> {
    let current_dir =
        env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let git_dir = current_dir.join(".git");

    if !git_dir.exists() {
        return Err("Not in a git repository. Please run 'git init' first.".to_string());
    }

    Ok(git_dir.join("git-pair"))
}

fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Error running git command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get current branch name".to_string());
    }

    let branch_name = String::from_utf8(output.stdout)
        .map_err(|e| format!("Error parsing branch name: {}", e))?
        .trim()
        .to_string();

    if branch_name.is_empty() {
        return Err("No branch name found (detached HEAD?)".to_string());
    }

    Ok(branch_name)
}

fn get_branch_config_file() -> Result<PathBuf, String> {
    let git_pair_dir = get_git_pair_dir()?;
    let branch_name = get_current_branch()?;

    // Sanitize branch name for filename (replace problematic characters)
    let safe_branch_name = branch_name.replace(['/', '\\', ':'], "_");

    Ok(git_pair_dir.join(format!("config-{}", safe_branch_name)))
}

// Global roster management functions
fn get_global_config_dir() -> Result<PathBuf, String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_dir = PathBuf::from(home_dir).join(".config").join("git-pair");
    Ok(config_dir)
}

fn get_global_roster_file() -> Result<PathBuf, String> {
    // Check for environment variable override (useful for testing)
    if let Ok(custom_path) = env::var("GIT_PAIR_ROSTER_FILE") {
        return Ok(PathBuf::from(custom_path));
    }

    let config_dir = get_global_config_dir()?;
    Ok(config_dir.join("roster"))
}

pub fn add_global_coauthor(alias: &str, name: &str, email: &str) -> Result<String, String> {
    let roster_file = get_global_roster_file()?;

    // Create parent directory if it doesn't exist (handle both default and custom paths)
    if let Some(parent) = roster_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Error creating roster directory: {}", e))?;
    }

    // Read existing roster or create default content
    let content = if roster_file.exists() {
        fs::read_to_string(&roster_file)
            .map_err(|e| format!("Error reading global roster: {}", e))?
    } else {
        "# Global git-pair roster\n# Format: alias|name|email\n".to_string()
    };

    // Check if alias already exists
    if content
        .lines()
        .any(|line| line.starts_with(&format!("{}|", alias)))
    {
        return Err(format!("Alias '{}' already exists in global roster", alias));
    }

    // Add new entry
    let new_entry = format!("{}|{}|{}\n", alias, name, email);
    let new_content = content + &new_entry;

    fs::write(&roster_file, new_content)
        .map_err(|e| format!("Error writing to global roster: {}", e))?;

    Ok(format!(
        "Added '{}' ({} <{}>) to global roster",
        alias, name, email
    ))
}

pub fn get_global_roster() -> Result<Vec<(String, String, String)>, String> {
    let roster_file = get_global_roster_file()?;

    if !roster_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&roster_file)
        .map_err(|e| format!("Error reading global roster: {}", e))?;

    let mut roster = Vec::new();
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 3 {
            roster.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
    }

    Ok(roster)
}

pub fn add_coauthor_from_global(alias: &str) -> Result<String, String> {
    let roster = get_global_roster()?;

    // Find the alias in the roster
    if let Some((_, name, email)) = roster.iter().find(|(a, _, _)| a == alias) {
        // Split name into first and last name for the existing add_coauthor function
        let name_parts: Vec<&str> = name.split_whitespace().collect();
        if name_parts.len() >= 2 {
            let first_name = name_parts[0];
            let last_name = name_parts[1..].join(" ");
            add_coauthor(first_name, &last_name, email)
        } else {
            // If only one name, use it as first name and empty last name
            add_coauthor(name, "", email)
        }
    } else {
        Err(format!("Alias '{}' not found in global roster. Use 'git pair list --global' to see available aliases.", alias))
    }
}

pub fn init_pair_config() -> Result<String, String> {
    let _current_dir =
        env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let git_pair_dir = get_git_pair_dir()?;
    let branch_name = get_current_branch()?;

    // Create .git/git-pair directory
    fs::create_dir_all(&git_pair_dir)
        .map_err(|e| format!("Error creating git-pair directory: {}", e))?;

    // Create branch-specific config file
    let config_file = get_branch_config_file()?;
    let default_config = format!(
        "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
        branch_name
    );

    if config_file.exists() {
        Ok(format!(
            "git-pair already initialized for branch '{}'",
            branch_name
        ))
    } else {
        fs::write(&config_file, default_config)
            .map_err(|e| format!("Error creating config file: {}", e))?;
        Ok(format!(
            "Successfully initialized git-pair for branch '{}'!\nConfiguration file created at: {}",
            branch_name,
            config_file.display()
        ))
    }
}

pub fn add_coauthor(name: &str, surname: &str, email: &str) -> Result<String, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;

    // Check if git-pair is initialized for this branch
    if !config_file.exists() {
        return Err(format!(
            "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
            branch_name
        ));
    }

    // Read existing config
    let existing_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;

    // Create the co-author entry
    let full_name = format!("{} {}", name, surname);
    let coauthor_line = format!("Co-authored-by: {} <{}>\n", full_name, email);

    // Check if this co-author already exists
    if existing_content.contains(coauthor_line.trim()) {
        return Ok(format!(
            "Co-author '{}' <{}> already exists on branch '{}'",
            full_name, email, branch_name
        ));
    }

    // Append the new co-author
    let new_content = existing_content + &coauthor_line;

    fs::write(&config_file, new_content)
        .map_err(|e| format!("Error writing to config file: {}", e))?;

    update_commit_template()?;
    Ok(format!(
        "Added co-author: {} <{}> to branch '{}'",
        full_name, email, branch_name
    ))
}

pub fn update_commit_template() -> Result<(), String> {
    let config_file = get_branch_config_file()?;

    // Read the config file to get co-authors
    let config_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;

    // Extract co-author lines
    let coauthor_lines: Vec<&str> = config_content
        .lines()
        .filter(|line| line.starts_with("Co-authored-by:"))
        .collect();

    if coauthor_lines.is_empty() {
        // No co-authors, remove the hook
        remove_git_hook()?;
    } else {
        // Install or update the hook with current co-authors
        let current_dir =
            env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
        install_git_hook_in(&current_dir)?;
    }

    Ok(())
}

fn remove_git_hook_in(working_dir: &Path) -> Result<(), String> {
    let hook_file = working_dir
        .join(".git")
        .join("hooks")
        .join("prepare-commit-msg");

    if hook_file.exists() {
        let hook_content = fs::read_to_string(&hook_file)
            .map_err(|e| format!("Error reading hook file: {}", e))?;

        // Check if our section exists
        if let Some(new_content) = remove_git_pair_section(&hook_content) {
            if is_effectively_empty(&new_content) {
                // If only whitespace/comments/shebang remain, remove the entire file
                fs::remove_file(&hook_file)
                    .map_err(|e| format!("Error removing git hook: {}", e))?;
            } else {
                // Write back the content without our section
                fs::write(&hook_file, new_content)
                    .map_err(|e| format!("Error updating git hook: {}", e))?;
            }
        }
    }

    Ok(())
}

fn remove_git_hook() -> Result<(), String> {
    let current_dir =
        env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    remove_git_hook_in(&current_dir)
}

pub fn remove_coauthor(identifier: &str) -> Result<String, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;

    // Check if git-pair is initialized for this branch
    if !config_file.exists() {
        return Err(format!(
            "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
            branch_name
        ));
    }

    // Read existing config
    let existing_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;

    // Get current co-authors
    let mut coauthor_lines: Vec<String> = existing_content
        .lines()
        .filter(|line| line.starts_with("Co-authored-by:"))
        .map(|line| line.to_string())
        .collect();

    // Store original count for comparison
    let original_count = coauthor_lines.len();

    // Try to match by different criteria
    coauthor_lines.retain(|line| !matches_coauthor(line, identifier));

    if coauthor_lines.len() == original_count {
        // No co-author was removed, check if it might be a global alias
        if let Ok(roster) = get_global_roster() {
            if let Some((_, name, email)) = roster.iter().find(|(alias, _, _)| alias == identifier)
            {
                // Try to remove by the actual name/email from the global roster
                let full_name_pattern = name;
                let email_pattern = email;

                coauthor_lines.retain(|line| {
                    !line.contains(full_name_pattern) && !line.contains(email_pattern)
                });

                if coauthor_lines.len() == original_count {
                    return Err(format!(
                        "Co-author matching alias '{}' ({} <{}>) not found on branch '{}'",
                        identifier, name, email, branch_name
                    ));
                }
            } else {
                return Err(format!("Co-author '{}' not found on branch '{}'. Use 'git-pair status' to see current co-authors.", identifier, branch_name));
            }
        } else {
            return Err(format!("Co-author '{}' not found on branch '{}'. Use 'git-pair status' to see current co-authors.", identifier, branch_name));
        }
    }

    // Reconstruct the config file content
    let mut new_content = String::new();

    // Add header
    new_content.push_str(&format!(
        "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
        branch_name
    ));

    // Add remaining co-authors
    for coauthor in &coauthor_lines {
        new_content.push_str(coauthor);
        new_content.push('\n');
    }

    // Write back the updated content
    fs::write(&config_file, new_content)
        .map_err(|e| format!("Error writing to config file: {}", e))?;

    // Update the commit template
    update_commit_template()?;

    let removed_count = original_count - coauthor_lines.len();
    if removed_count == 1 {
        Ok(format!(
            "Removed 1 co-author matching '{}' from branch '{}'",
            identifier, branch_name
        ))
    } else {
        Ok(format!(
            "Removed {} co-authors matching '{}' from branch '{}'",
            removed_count, identifier, branch_name
        ))
    }
}

fn matches_coauthor(coauthor_line: &str, identifier: &str) -> bool {
    // Match by full name (case-insensitive)
    if coauthor_line
        .to_lowercase()
        .contains(&identifier.to_lowercase())
    {
        return true;
    }

    // Match by email
    if coauthor_line.contains(identifier) {
        return true;
    }

    false
}

pub fn clear_coauthors() -> Result<String, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;

    // Check if git-pair is initialized for this branch
    if !config_file.exists() {
        return Err(format!(
            "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
            branch_name
        ));
    }

    // Reset config file to default content
    let default_config = format!(
        "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
        branch_name
    );
    fs::write(&config_file, default_config)
        .map_err(|e| format!("Error clearing config file: {}", e))?;

    // Remove git hook
    remove_git_hook()?;

    Ok(format!(
        "Cleared all co-authors for branch '{}' and uninstalled git hook",
        branch_name
    ))
}

pub fn get_coauthors() -> Result<Vec<String>, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;

    if !config_file.exists() {
        return Err(format!(
            "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
            branch_name
        ));
    }

    let config_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;

    let coauthors: Vec<String> = config_content
        .lines()
        .filter(|line| line.starts_with("Co-authored-by:"))
        .map(|line| line.to_string())
        .collect();

    Ok(coauthors)
}

// Helper functions for hook management

/// Checks if hook content is effectively empty (only shebang, whitespace, or comments)
fn is_effectively_empty(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            return false;
        }
    }
    true
}

/// Merges git-pair section into existing hook content
fn merge_git_pair_section(
    existing_content: &str,
    git_pair_section: &str,
) -> Result<String, String> {
    const BEGIN_MARKER: &str = "# BEGIN git-pair";
    const END_MARKER: &str = "# END git-pair";

    // Check if git-pair section already exists
    if let Some(begin_pos) = existing_content.find(BEGIN_MARKER) {
        if let Some(end_pos) = existing_content.find(END_MARKER) {
            // Replace existing git-pair section
            let before = &existing_content[..begin_pos];
            let after = &existing_content[end_pos + END_MARKER.len()..];

            // Remove trailing newline from 'before' if it exists, and ensure proper spacing
            let before_trimmed = before.trim_end();
            let after_trimmed = after.trim_start();

            let mut result = String::new();

            // Add shebang if this is a new file and before section is empty
            if before_trimmed.is_empty() && !existing_content.starts_with("#!") {
                result.push_str("#!/bin/sh\n");
            }

            if !before_trimmed.is_empty() {
                result.push_str(before_trimmed);
                result.push('\n');
            }

            result.push_str(git_pair_section);

            if !after_trimmed.is_empty() {
                result.push('\n');
                result.push_str(after_trimmed);
            }

            Ok(result)
        } else {
            Err("Found BEGIN marker but no END marker in existing hook".to_string())
        }
    } else {
        // Append git-pair section to existing content
        let mut result = String::new();

        if existing_content.trim().is_empty() {
            // New file, add shebang
            result.push_str("#!/bin/sh\n");
            result.push_str(git_pair_section);
        } else {
            // Existing content, append our section
            result.push_str(existing_content.trim_end());
            result.push_str("\n\n");
            result.push_str(git_pair_section);
        }

        Ok(result)
    }
}

/// Removes git-pair section from hook content, returns None if no section found
fn remove_git_pair_section(content: &str) -> Option<String> {
    const BEGIN_MARKER: &str = "# BEGIN git-pair";
    const END_MARKER: &str = "# END git-pair";

    if let Some(begin_pos) = content.find(BEGIN_MARKER) {
        if let Some(end_pos) = content.find(END_MARKER) {
            let before = &content[..begin_pos];
            let after = &content[end_pos + END_MARKER.len()..];

            // Clean up spacing - remove extra newlines around the removed section
            let before_trimmed = before.trim_end();
            let after_trimmed = after.trim_start();

            let mut result = String::new();

            if !before_trimmed.is_empty() {
                result.push_str(before_trimmed);
                if !after_trimmed.is_empty() {
                    result.push('\n');
                }
            }

            if !after_trimmed.is_empty() {
                result.push_str(after_trimmed);
            }

            Some(result)
        } else {
            // Found BEGIN but no END, don't modify
            None
        }
    } else {
        // No git-pair section found
        None
    }
}

fn install_git_hook_in(working_dir: &Path) -> Result<(), String> {
    let hooks_dir = working_dir.join(".git").join("hooks");
    let hook_file = hooks_dir.join("prepare-commit-msg");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir).map_err(|e| format!("Error creating hooks directory: {}", e))?;

    // Read existing hook content if it exists
    let existing_content = if hook_file.exists() {
        fs::read_to_string(&hook_file)
            .map_err(|e| format!("Error reading existing hook file: {}", e))?
    } else {
        String::new()
    };

    // Generate our git-pair hook section
    let git_pair_section = r#"# BEGIN git-pair
# git-pair hook to automatically add co-authors

COMMIT_MSG_FILE=$1
COMMIT_SOURCE=$2

# Only add co-authors for regular commits (not merges, rebases, etc.)
if [ -z "$COMMIT_SOURCE" ] || [ "$COMMIT_SOURCE" = "message" ]; then
  # Check if co-authors are already present
  if ! grep -q "Co-authored-by:" "$COMMIT_MSG_FILE"; then
    # Get current branch and config file
    CURRENT_BRANCH=$(git branch --show-current)
    SAFE_BRANCH=$(echo "$CURRENT_BRANCH" | sed 's/[/\\:]/_/g')
    CONFIG_FILE=".git/git-pair/config-$SAFE_BRANCH"

    # Add co-authors from branch-specific config if it exists
    if [ -f "$CONFIG_FILE" ]; then
      COAUTHORS=$(grep '^Co-authored-by:' "$CONFIG_FILE")
      if [ -n "$COAUTHORS" ]; then
        echo "" >> "$COMMIT_MSG_FILE"
        echo "$COAUTHORS" >> "$COMMIT_MSG_FILE"
      fi
    fi
  fi
fi
# END git-pair"#;

    // Create the new hook content
    let new_content = merge_git_pair_section(&existing_content, git_pair_section)?;

    // Write the hook file
    fs::write(&hook_file, new_content).map_err(|e| format!("Error writing git hook: {}", e))?;

    // Make the hook executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_file)
            .map_err(|e| format!("Error getting hook file permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_file, perms)
            .map_err(|e| format!("Error setting hook file permissions: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Import the helper function for tests
    use super::matches_coauthor;

    // Mutex to ensure global roster tests don't interfere with each other
    static GLOBAL_ROSTER_TEST_LOCK: Mutex<()> = Mutex::new(());

    // Simple RAII wrapper for temporary directories
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> std::io::Result<Self> {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let mut temp_path = env::temp_dir();
            temp_path.push(format!(
                "git-pair-test-{}-{}",
                std::process::id(),
                timestamp
            ));

            fs::create_dir_all(&temp_path)?;
            Ok(TempDir { path: temp_path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    // Simple temporary file helper
    fn create_temp_file() -> std::io::Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut temp_path = env::temp_dir();
        temp_path.push(format!(
            "git-pair-test-{}-{}",
            std::process::id(),
            timestamp
        ));

        // Create empty file
        fs::write(&temp_path, "")?;
        Ok(temp_path)
    }

    // Test helper functions that work with a specific directory instead of changing global cwd
    fn get_git_pair_dir_in(working_dir: &Path) -> Result<PathBuf, String> {
        let git_dir = working_dir.join(".git");

        if !git_dir.exists() {
            return Err("Not in a git repository. Please run 'git init' first.".to_string());
        }

        Ok(git_dir.join("git-pair"))
    }

    fn get_current_branch_in(working_dir: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(working_dir)
            .output()
            .map_err(|e| format!("Error running git command: {}", e))?;

        if !output.status.success() {
            return Err("Error getting current branch".to_string());
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            return Err("No current branch found".to_string());
        }

        Ok(branch)
    }

    fn get_branch_config_file_in(working_dir: &Path) -> Result<PathBuf, String> {
        let git_pair_dir = get_git_pair_dir_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        // Sanitize branch name for filename (replace problematic characters)
        let safe_branch_name = branch_name.replace(['/', '\\', ':'], "_");

        Ok(git_pair_dir.join(format!("config-{}", safe_branch_name)))
    }

    fn init_pair_config_in(working_dir: &Path) -> Result<String, String> {
        let git_pair_dir = get_git_pair_dir_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        // Create .git/git-pair directory
        fs::create_dir_all(&git_pair_dir)
            .map_err(|e| format!("Error creating git-pair directory: {}", e))?;

        // Create branch-specific config file
        let config_file = get_branch_config_file_in(working_dir)?;
        let default_config = format!(
            "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
            branch_name
        );

        if config_file.exists() {
            Ok(format!(
                "git-pair already initialized for branch '{}'",
                branch_name
            ))
        } else {
            fs::write(&config_file, default_config)
                .map_err(|e| format!("Error creating config file: {}", e))?;
            Ok(format!("Successfully initialized git-pair for branch '{}'!\nConfiguration file created at: {}", branch_name, config_file.display()))
        }
    }

    fn add_coauthor_in(
        working_dir: &Path,
        name: &str,
        surname: &str,
        email: &str,
    ) -> Result<String, String> {
        let config_file = get_branch_config_file_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        // Check if git-pair is initialized for this branch
        if !config_file.exists() {
            return Err(format!(
                "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
                branch_name
            ));
        }

        // Read existing config
        let existing_content = fs::read_to_string(&config_file)
            .map_err(|e| format!("Error reading config file: {}", e))?;

        // Create the co-author entry
        let full_name = format!("{} {}", name, surname);
        let coauthor_line = format!("Co-authored-by: {} <{}>\n", full_name, email);

        // Check if this co-author already exists
        if existing_content.contains(coauthor_line.trim()) {
            return Ok(format!(
                "Co-author '{}' <{}> already exists on branch '{}'",
                full_name, email, branch_name
            ));
        }

        // Append the new co-author
        let new_content = existing_content + &coauthor_line;

        fs::write(&config_file, new_content)
            .map_err(|e| format!("Error writing to config file: {}", e))?;

        // Install/update hook
        install_git_hook_in(working_dir)?;
        Ok(format!(
            "Added co-author: {} <{}> to branch '{}'",
            full_name, email, branch_name
        ))
    }

    fn get_coauthors_in(working_dir: &Path) -> Result<Vec<String>, String> {
        let config_file = get_branch_config_file_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        if !config_file.exists() {
            return Err(format!(
                "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
                branch_name
            ));
        }

        let content = fs::read_to_string(&config_file)
            .map_err(|e| format!("Error reading config file: {}", e))?;

        let coauthors: Vec<String> = content
            .lines()
            .filter(|line| line.starts_with("Co-authored-by:"))
            .map(|line| line.to_string())
            .collect();

        Ok(coauthors)
    }

    fn remove_coauthor_in(working_dir: &Path, identifier: &str) -> Result<String, String> {
        let config_file = get_branch_config_file_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        // Check if git-pair is initialized for this branch
        if !config_file.exists() {
            return Err(format!(
                "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
                branch_name
            ));
        }

        // Read existing config
        let existing_content = fs::read_to_string(&config_file)
            .map_err(|e| format!("Error reading config file: {}", e))?;

        // Get current co-authors
        let mut coauthor_lines: Vec<String> = existing_content
            .lines()
            .filter(|line| line.starts_with("Co-authored-by:"))
            .map(|line| line.to_string())
            .collect();

        // Store original count for comparison
        let original_count = coauthor_lines.len();

        // Try to match by different criteria
        coauthor_lines.retain(|line| !matches_coauthor(line, identifier));

        if coauthor_lines.len() == original_count {
            // No co-author was removed, check if it might be a global alias
            if let Ok(roster) = get_global_roster() {
                if let Some((_, name, email)) =
                    roster.iter().find(|(alias, _, _)| alias == identifier)
                {
                    // Try to remove by the actual name/email from the global roster
                    let full_name_pattern = name;
                    let email_pattern = email;

                    coauthor_lines.retain(|line| {
                        !line.contains(full_name_pattern) && !line.contains(email_pattern)
                    });

                    if coauthor_lines.len() == original_count {
                        return Err(format!(
                            "Co-author matching alias '{}' ({} <{}>) not found on branch '{}'",
                            identifier, name, email, branch_name
                        ));
                    }
                } else {
                    return Err(format!("Co-author '{}' not found on branch '{}'. Use 'git-pair status' to see current co-authors.", identifier, branch_name));
                }
            } else {
                return Err(format!("Co-author '{}' not found on branch '{}'. Use 'git-pair status' to see current co-authors.", identifier, branch_name));
            }
        }

        // Reconstruct the config file content
        let mut new_content = String::new();

        // Add header
        new_content.push_str(&format!(
            "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
            branch_name
        ));

        // Add remaining co-authors
        for coauthor in &coauthor_lines {
            new_content.push_str(coauthor);
            new_content.push('\n');
        }

        // Write back the updated content
        fs::write(&config_file, new_content)
            .map_err(|e| format!("Error writing to config file: {}", e))?;

        // Update git hook
        if coauthor_lines.is_empty() {
            remove_git_hook_in(working_dir)?;
        } else {
            install_git_hook_in(working_dir)?;
        }

        let removed_count = original_count - coauthor_lines.len();
        if removed_count == 1 {
            Ok(format!(
                "Removed 1 co-author matching '{}' from branch '{}'",
                identifier, branch_name
            ))
        } else {
            Ok(format!(
                "Removed {} co-authors matching '{}' from branch '{}'",
                removed_count, identifier, branch_name
            ))
        }
    }

    fn clear_coauthors_in(working_dir: &Path) -> Result<String, String> {
        let config_file = get_branch_config_file_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        // Check if git-pair is initialized for this branch
        if !config_file.exists() {
            return Err(format!(
                "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
                branch_name
            ));
        }

        // Reset config file to default content
        let default_config = format!(
            "# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n",
            branch_name
        );
        fs::write(&config_file, default_config)
            .map_err(|e| format!("Error clearing config file: {}", e))?;

        // Remove git hook
        remove_git_hook_in(working_dir)?;

        Ok(format!(
            "Cleared all co-authors for branch '{}' and uninstalled git hook",
            branch_name
        ))
    }

    fn add_coauthor_from_global_in(working_dir: &Path, alias: &str) -> Result<String, String> {
        // Check if git-pair is initialized
        let config_file = get_branch_config_file_in(working_dir)?;
        let branch_name = get_current_branch_in(working_dir)?;

        if !config_file.exists() {
            return Err(format!(
                "git-pair not initialized for branch '{}'. Please run 'git-pair init' first.",
                branch_name
            ));
        }

        // Get from global roster
        let roster = get_global_roster()?;
        let (_, name, email) = roster
            .iter()
            .find(|(a, _, _)| a == alias)
            .ok_or_else(|| format!("Alias '{}' not found in global roster", alias))?;

        // Split name into first and last name
        let name_parts: Vec<&str> = name.split_whitespace().collect();
        let first_name = name_parts.first().map_or("", |v| v).to_string();
        let last_name = if name_parts.len() > 1 {
            name_parts[1..].join(" ")
        } else {
            String::new()
        };

        // Add the coauthor
        add_coauthor_in(working_dir, &first_name, &last_name, email)
    }

    // Test helper to create a temporary git repository without changing global cwd
    fn setup_test_repo() -> std::io::Result<TempDir> {
        use std::process::Command;
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize git repo in the temp directory (without changing global cwd)
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        // Configure git user (required for commits)
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()?;

        Ok(temp_dir)
    }

    #[test]
    fn test_init_pair_config_success() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");

        let result = init_pair_config_in(temp_dir.path()).expect("Init should succeed");
        assert!(result.contains("Successfully initialized git-pair for branch"));

        // Check that files were created
        assert!(temp_dir.path().join(".git/git-pair").exists());

        // Check branch-specific config file exists (should be config-main for default branch)
        let branch_config =
            get_branch_config_file_in(temp_dir.path()).expect("Should get branch config file");
        assert!(branch_config.exists());

        // Check config file content
        let config_content = fs::read_to_string(&branch_config).expect("Config file should exist");
        assert!(config_content.contains("# git-pair configuration file for branch"));
    }

    #[test]
    fn test_init_pair_config_already_initialized() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");

        // Initialize once
        init_pair_config_in(temp_dir.path()).expect("First init should succeed");

        // Initialize again
        let result = init_pair_config_in(temp_dir.path()).expect("Second init should succeed");
        assert!(result.contains("git-pair already initialized"));
    }

    #[test]
    fn test_init_pair_config_not_git_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let result = init_pair_config_in(temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not in a git repository"));
    }

    #[test]
    fn test_add_coauthor_success() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        let result = add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");
        assert!(result.contains("Added co-author: John Doe"));

        // Check branch-specific config file was updated
        let branch_name = get_current_branch_in(test_dir).expect("Should get current branch");
        let config_dir = test_dir.join(".git/git-pair");
        let config_file = config_dir.join(format!("config-{}", branch_name));
        let config_content = fs::read_to_string(&config_file).expect("Config file should exist");
        assert!(config_content.contains("Co-authored-by: John Doe <john.doe@example.com>"));

        // Check git hook was installed
        assert!(test_dir.join(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_add_coauthor_duplicate() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add first time
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("First add should succeed");

        // Add same person again
        let result = add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Duplicate add should succeed");
        assert!(result.contains("already exists"));
    }

    #[test]
    fn test_add_coauthor_not_initialized() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        let result = add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_multiple_coauthors() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add multiple co-authors
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("First add should succeed");
        add_coauthor_in(test_dir, "Jane", "Smith", "jane.smith@example.com")
            .expect("Second add should succeed");

        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 2);
        assert!(coauthors.iter().any(|c| c.contains("John Doe")));
        assert!(coauthors.iter().any(|c| c.contains("Jane Smith")));
    }

    #[test]
    fn test_clear_coauthors_success() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");

        let result = clear_coauthors_in(test_dir).expect("Clear should succeed");
        assert!(result.contains("Cleared all co-authors"));

        // Check that co-authors were cleared
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());

        // Check that git hook was removed
        assert!(!test_dir.join(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_clear_coauthors_not_initialized() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        let result = clear_coauthors_in(test_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_get_coauthors_empty() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());
    }

    #[test]
    fn test_get_coauthors_not_initialized() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        let result = get_coauthors_in(test_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_git_hook_functionality() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "Alice", "Johnson", "alice@example.com")
            .expect("Add should succeed");

        // Create a test file and commit with -m flag
        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, "test content").expect("Should write test file");

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(test_dir)
            .output()
            .expect("Git add should succeed");

        let output = Command::new("git")
            .args(["commit", "-m", "Test commit message"])
            .current_dir(test_dir)
            .output()
            .expect("Git commit should succeed");

        assert!(output.status.success());

        // Check that the commit message includes co-author
        let log_output = Command::new("git")
            .args(["log", "--pretty=format:%B", "-1"])
            .current_dir(test_dir)
            .output()
            .expect("Git log should succeed");

        let commit_message =
            String::from_utf8(log_output.stdout).expect("Log output should be valid UTF-8");
        assert!(commit_message.contains("Test commit message"));
        assert!(commit_message.contains("Co-authored-by: Alice Johnson <alice@example.com>"));
    }

    #[test]
    fn test_git_config_integration() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");

        // Check that git hook was installed
        assert!(test_dir.join(".git/hooks/prepare-commit-msg").exists());

        let hook_content = fs::read_to_string(test_dir.join(".git/hooks/prepare-commit-msg"))
            .expect("Hook file should exist");
        assert!(hook_content.contains("git-pair hook"));
        // With per-branch config, co-author names are read dynamically from config files
        // so they won't be hard-coded in the hook
        assert!(hook_content.contains("CONFIG_FILE"));
        assert!(hook_content.contains("grep '^Co-authored-by:'"));

        // Check that the branch-specific config file contains the co-author
        let branch_config =
            get_branch_config_file_in(test_dir).expect("Should get branch config file");
        let config_content = fs::read_to_string(&branch_config).expect("Config file should exist");
        assert!(config_content.contains("John Doe"));

        // Clear and check hook was removed
        clear_coauthors_in(test_dir).expect("Clear should succeed");

        // Hook should be removed
        assert!(!test_dir.join(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_global_roster_add_and_list() {
        let _lock = GLOBAL_ROSTER_TEST_LOCK.lock().unwrap();

        // Use a temporary roster file for testing
        let temp_path = create_temp_file().expect("Failed to create temp file");
        env::set_var("GIT_PAIR_ROSTER_FILE", temp_path.to_str().unwrap());

        // Test adding to global roster
        let result = add_global_coauthor("alice", "Alice Johnson", "alice@example.com")
            .expect("Should add to global roster");
        assert!(result.contains("Added 'alice'"));

        // Test listing global roster
        let roster = get_global_roster().expect("Should get global roster");
        assert_eq!(roster.len(), 1);
        assert_eq!(
            roster[0],
            (
                "alice".to_string(),
                "Alice Johnson".to_string(),
                "alice@example.com".to_string()
            )
        );

        // Test duplicate alias
        let result = add_global_coauthor("alice", "Alice Smith", "alice.smith@example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        // Clean up
        env::remove_var("GIT_PAIR_ROSTER_FILE");
    }

    #[test]
    fn test_add_coauthor_from_global() {
        let _lock = GLOBAL_ROSTER_TEST_LOCK.lock().unwrap();

        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        let temp_path = create_temp_file().expect("Failed to create temp file");
        env::set_var("GIT_PAIR_ROSTER_FILE", temp_path.to_str().unwrap());

        // Initialize branch config
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add to global roster
        add_global_coauthor("bob", "Bob Wilson", "bob@example.com")
            .expect("Should add to global roster");

        // Test adding from global roster
        let result =
            add_coauthor_from_global_in(test_dir, "bob").expect("Should add from global roster");
        assert!(result.contains("Added co-author: Bob Wilson"));

        // Verify it was added to branch config
        let coauthors = get_coauthors_in(test_dir).expect("Should get coauthors");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("Bob Wilson"));

        // Test non-existent alias
        let result = add_coauthor_from_global_in(test_dir, "charlie");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in global roster"));

        // Clean up
        env::remove_var("GIT_PAIR_ROSTER_FILE");
    }

    #[test]
    fn test_global_roster_empty() {
        let _lock = GLOBAL_ROSTER_TEST_LOCK.lock().unwrap();

        let temp_path = create_temp_file().expect("Failed to create temp file");
        env::set_var("GIT_PAIR_ROSTER_FILE", temp_path.to_str().unwrap());

        // Remove the file to test truly empty state
        fs::remove_file(&temp_path).ok();

        // Test empty global roster
        let roster = get_global_roster().expect("Should get empty global roster");
        assert!(roster.is_empty());

        // Clean up
        env::remove_var("GIT_PAIR_ROSTER_FILE");
    }

    #[test]
    fn test_remove_coauthor_by_name() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add multiple co-authors
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");
        add_coauthor_in(test_dir, "Jane", "Smith", "jane.smith@example.com")
            .expect("Add should succeed");

        // Remove by name
        let result = remove_coauthor_in(test_dir, "John Doe").expect("Remove should succeed");
        assert!(result.contains("Removed 1 co-author matching 'John Doe'"));

        // Check remaining co-authors
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("Jane Smith"));
        assert!(!coauthors[0].contains("John Doe"));
    }

    #[test]
    fn test_remove_coauthor_by_email() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add multiple co-authors
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");
        add_coauthor_in(test_dir, "Jane", "Smith", "jane.smith@example.com")
            .expect("Add should succeed");

        // Remove by email
        let result =
            remove_coauthor_in(test_dir, "jane.smith@example.com").expect("Remove should succeed");
        assert!(result.contains("Removed 1 co-author matching 'jane.smith@example.com'"));

        // Check remaining co-authors
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("John Doe"));
        assert!(!coauthors[0].contains("Jane Smith"));
    }

    #[test]
    fn test_remove_coauthor_by_global_alias() {
        let _lock = GLOBAL_ROSTER_TEST_LOCK.lock().unwrap();

        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        let temp_path = create_temp_file().expect("Failed to create temp file");
        env::set_var("GIT_PAIR_ROSTER_FILE", temp_path.to_str().unwrap());

        // Setup global roster
        add_global_coauthor("alice", "Alice Johnson", "alice@example.com")
            .expect("Should add to global roster");
        add_global_coauthor("bob", "Bob Wilson", "bob@example.com")
            .expect("Should add to global roster");

        // Initialize and add co-authors
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_from_global_in(test_dir, "alice").expect("Add alice should succeed");
        add_coauthor_from_global_in(test_dir, "bob").expect("Add bob should succeed");

        // Remove by alias
        let result = remove_coauthor_in(test_dir, "alice").expect("Remove should succeed");
        assert!(result.contains("Removed 1 co-author matching 'alice'"));

        // Check remaining co-authors
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("Bob Wilson"));
        assert!(!coauthors[0].contains("Alice Johnson"));

        // Clean up
        env::remove_var("GIT_PAIR_ROSTER_FILE");
    }

    #[test]
    fn test_remove_coauthor_not_found() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add one co-author
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");

        // Try to remove non-existent co-author
        let result = remove_coauthor_in(test_dir, "Jane Smith");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found on branch"));

        // Verify original co-author is still there
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("John Doe"));
    }

    #[test]
    fn test_remove_coauthor_not_initialized() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        let result = remove_coauthor_in(test_dir, "John Doe");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_remove_last_coauthor_removes_hook() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add one co-author
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");

        // Verify hook exists
        assert!(test_dir.join(".git/hooks/prepare-commit-msg").exists());

        // Remove the only co-author
        remove_coauthor_in(test_dir, "John Doe").expect("Remove should succeed");

        // Verify hook was removed since no co-authors remain
        assert!(!test_dir.join(".git/hooks/prepare-commit-msg").exists());

        // Verify no co-authors remain
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());
    }

    #[test]
    fn test_remove_coauthor_case_insensitive() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();
        init_pair_config_in(test_dir).expect("Init should succeed");

        // Add co-author
        add_coauthor_in(test_dir, "John", "Doe", "john.doe@example.com")
            .expect("Add should succeed");

        // Remove with different case
        let result = remove_coauthor_in(test_dir, "john doe").expect("Remove should succeed");
        assert!(result.contains("Removed 1 co-author matching 'john doe'"));

        // Verify co-author was removed
        let coauthors = get_coauthors_in(test_dir).expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());
    }

    // Tests for improved hook management

    #[test]
    fn test_is_effectively_empty() {
        assert!(is_effectively_empty(""));
        assert!(is_effectively_empty("   \n  \t  "));
        assert!(is_effectively_empty("#!/bin/sh"));
        assert!(is_effectively_empty("#!/bin/sh\n# comment"));
        assert!(is_effectively_empty("#!/bin/sh\n\n# comment\n   "));
        assert!(!is_effectively_empty("#!/bin/sh\necho 'something'"));
        assert!(!is_effectively_empty("echo 'test'"));
    }

    #[test]
    fn test_merge_git_pair_section_new_file() {
        let existing = "";
        let git_pair_section = "# BEGIN git-pair\necho 'git-pair'\n# END git-pair";

        let result = merge_git_pair_section(existing, git_pair_section).unwrap();
        assert!(result.starts_with("#!/bin/sh\n"));
        assert!(result.contains("# BEGIN git-pair"));
        assert!(result.contains("# END git-pair"));
    }

    #[test]
    fn test_merge_git_pair_section_existing_content() {
        let existing = "#!/bin/sh\necho 'existing hook'";
        let git_pair_section = "# BEGIN git-pair\necho 'git-pair'\n# END git-pair";

        let result = merge_git_pair_section(existing, git_pair_section).unwrap();
        assert!(result.contains("echo 'existing hook'"));
        assert!(result.contains("# BEGIN git-pair"));
        assert!(result.ends_with("# END git-pair"));
    }

    #[test]
    fn test_merge_git_pair_section_replace_existing() {
        let existing =
            "#!/bin/sh\necho 'before'\n# BEGIN git-pair\necho 'old'\n# END git-pair\necho 'after'";
        let git_pair_section = "# BEGIN git-pair\necho 'new'\n# END git-pair";

        let result = merge_git_pair_section(existing, git_pair_section).unwrap();
        assert!(result.contains("echo 'before'"));
        assert!(result.contains("echo 'new'"));
        assert!(result.contains("echo 'after'"));
        assert!(!result.contains("echo 'old'"));
    }

    #[test]
    fn test_remove_git_pair_section_success() {
        let content = "#!/bin/sh\necho 'before'\n# BEGIN git-pair\necho 'git-pair'\n# END git-pair\necho 'after'";

        let result = remove_git_pair_section(content).unwrap();
        assert!(result.contains("echo 'before'"));
        assert!(result.contains("echo 'after'"));
        assert!(!result.contains("git-pair"));
        assert!(!result.contains("BEGIN"));
    }

    #[test]
    fn test_remove_git_pair_section_not_found() {
        let content = "#!/bin/sh\necho 'no git-pair here'";

        let result = remove_git_pair_section(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_git_pair_section_only_content() {
        let content = "#!/bin/sh\n# BEGIN git-pair\necho 'git-pair'\n# END git-pair";

        let result = remove_git_pair_section(content).unwrap();
        assert_eq!(result.trim(), "#!/bin/sh");
    }

    #[test]
    fn test_hook_preservation_workflow() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        // Create an existing hook
        let hooks_dir = test_dir.join(".git/hooks");
        fs::create_dir_all(&hooks_dir).expect("Should create hooks dir");
        let hook_file = hooks_dir.join("prepare-commit-msg");
        let existing_hook = "#!/bin/sh\necho 'existing hook logic'";
        fs::write(&hook_file, existing_hook).expect("Should write existing hook");

        // Initialize git-pair and add co-author
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "John", "Doe", "john@example.com").expect("Add should succeed");

        // Check that existing hook content is preserved
        let hook_content = fs::read_to_string(&hook_file).expect("Hook should exist");
        assert!(hook_content.contains("existing hook logic"));
        assert!(hook_content.contains("# BEGIN git-pair"));
        assert!(hook_content.contains("# END git-pair"));

        // Clear co-authors and check that only git-pair section is removed
        clear_coauthors_in(test_dir).expect("Clear should succeed");

        let remaining_content = fs::read_to_string(&hook_file).expect("Hook should still exist");
        assert!(remaining_content.contains("existing hook logic"));
        assert!(!remaining_content.contains("git-pair"));
    }

    #[test]
    fn test_hook_complete_removal() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        // Initialize git-pair and add co-author (no existing hook)
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "John", "Doe", "john@example.com").expect("Add should succeed");

        let hook_file = test_dir.join(".git/hooks/prepare-commit-msg");
        assert!(hook_file.exists());

        // Clear co-authors - should remove entire hook file since it only contains git-pair content
        clear_coauthors_in(test_dir).expect("Clear should succeed");

        assert!(!hook_file.exists());
    }

    #[test]
    fn test_hook_update_preserves_existing() {
        let temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let test_dir = temp_dir.path();

        // Create existing hook
        let hooks_dir = test_dir.join(".git/hooks");
        fs::create_dir_all(&hooks_dir).expect("Should create hooks dir");
        let hook_file = hooks_dir.join("prepare-commit-msg");
        fs::write(&hook_file, "#!/bin/sh\necho 'original'").expect("Should write hook");

        // Initialize and add first co-author
        init_pair_config_in(test_dir).expect("Init should succeed");
        add_coauthor_in(test_dir, "John", "Doe", "john@example.com").expect("Add should succeed");

        // Add second co-author (should update hook)
        add_coauthor_in(test_dir, "Jane", "Smith", "jane@example.com").expect("Add should succeed");

        // Original content should still be there
        let hook_content = fs::read_to_string(&hook_file).expect("Hook should exist");
        assert!(hook_content.contains("echo 'original'"));
        assert!(hook_content.contains("git-pair"));

        // Should only have one git-pair section
        assert_eq!(hook_content.matches("# BEGIN git-pair").count(), 1);
        assert_eq!(hook_content.matches("# END git-pair").count(), 1);
    }
}
