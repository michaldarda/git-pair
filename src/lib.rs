use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn get_git_pair_dir() -> Result<PathBuf, String> {
    let current_dir = env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let git_dir = current_dir.join(".git");
    
    if !git_dir.exists() {
        return Err("Not in a git repository. Please run 'git init' first.".to_string());
    }
    
    Ok(git_dir.join("git-pair"))
}

fn get_current_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Error getting current branch: {}", e))?;
    
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
    let safe_branch_name = branch_name
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_");
    
    Ok(git_pair_dir.join(format!("config-{}", safe_branch_name)))
}

// Global roster management functions
fn get_global_config_dir() -> Result<PathBuf, String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_dir = PathBuf::from(home_dir).join(".config").join("git-pair");
    Ok(config_dir)
}

fn get_global_roster_file() -> Result<PathBuf, String> {
    let config_dir = get_global_config_dir()?;
    Ok(config_dir.join("roster"))
}

pub fn add_global_coauthor(alias: &str, name: &str, email: &str) -> Result<String, String> {
    let config_dir = get_global_config_dir()?;
    let roster_file = get_global_roster_file()?;
    
    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir).map_err(|e| format!("Error creating global config directory: {}", e))?;
    
    // Read existing roster or create default content
    let content = if roster_file.exists() {
        fs::read_to_string(&roster_file)
            .map_err(|e| format!("Error reading global roster: {}", e))?
    } else {
        "# Global git-pair roster\n# Format: alias|name|email\n".to_string()
    };
    
    // Check if alias already exists
    if content.lines().any(|line| {
        line.starts_with(&format!("{}|", alias))
    }) {
        return Err(format!("Alias '{}' already exists in global roster", alias));
    }
    
    // Add new entry
    let new_entry = format!("{}|{}|{}\n", alias, name, email);
    let new_content = content + &new_entry;
    
    fs::write(&roster_file, new_content)
        .map_err(|e| format!("Error writing to global roster: {}", e))?;
    
    Ok(format!("Added '{}' ({} <{}>) to global roster", alias, name, email))
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
            roster.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
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
    let _current_dir = env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let git_pair_dir = get_git_pair_dir()?;
    let branch_name = get_current_branch()?;
    
    // Create .git/git-pair directory
    fs::create_dir_all(&git_pair_dir).map_err(|e| format!("Error creating git-pair directory: {}", e))?;
    
    // Create branch-specific config file
    let config_file = get_branch_config_file()?;
    let default_config = format!("# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n", branch_name);
    
    if config_file.exists() {
        Ok(format!("git-pair already initialized for branch '{}'", branch_name))
    } else {
        fs::write(&config_file, default_config)
            .map_err(|e| format!("Error creating config file: {}", e))?;
        Ok(format!("Successfully initialized git-pair for branch '{}'!\nConfiguration file created at: {}", branch_name, config_file.display()))
    }
}

pub fn add_coauthor(name: &str, surname: &str, email: &str) -> Result<String, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;
    
    // Check if git-pair is initialized for this branch
    if !config_file.exists() {
        return Err(format!("git-pair not initialized for branch '{}'. Please run 'git-pair init' first.", branch_name));
    }
    
    // Read existing config
    let existing_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;
    
    // Create the co-author entry
    let full_name = format!("{} {}", name, surname);
    let coauthor_line = format!("Co-authored-by: {} <{}>\n", full_name, email);
    
    // Check if this co-author already exists
    if existing_content.contains(&coauthor_line.trim()) {
        return Ok(format!("Co-author '{}' <{}> already exists on branch '{}'", full_name, email, branch_name));
    }
    
    // Append the new co-author
    let new_content = existing_content + &coauthor_line;
    
    fs::write(&config_file, new_content)
        .map_err(|e| format!("Error writing to config file: {}", e))?;
    
    update_commit_template()?;
    Ok(format!("Added co-author: {} <{}> to branch '{}'", full_name, email, branch_name))
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
        install_git_hook(&coauthor_lines)?;
    }
    
    Ok(())
}

fn install_git_hook(_coauthor_lines: &[&str]) -> Result<(), String> {
    let current_dir = env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let hooks_dir = current_dir.join(".git").join("hooks");
    let hook_file = hooks_dir.join("prepare-commit-msg");
    
    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir).map_err(|e| format!("Error creating hooks directory: {}", e))?;
    
    // Create the hook script that dynamically reads branch-specific config
    let mut hook_content = String::new();
    hook_content.push_str("#!/bin/sh\n");
    hook_content.push_str("# git-pair hook to automatically add co-authors\n\n");
    hook_content.push_str("COMMIT_MSG_FILE=$1\n");
    hook_content.push_str("COMMIT_SOURCE=$2\n\n");
    hook_content.push_str("# Only add co-authors for regular commits (not merges, rebases, etc.)\n");
    hook_content.push_str("if [ -z \"$COMMIT_SOURCE\" ] || [ \"$COMMIT_SOURCE\" = \"message\" ]; then\n");
    hook_content.push_str("  # Check if co-authors are already present\n");
    hook_content.push_str("  if ! grep -q \"Co-authored-by:\" \"$COMMIT_MSG_FILE\"; then\n");
    hook_content.push_str("    # Get current branch and config file\n");
    hook_content.push_str("    CURRENT_BRANCH=$(git branch --show-current)\n");
    hook_content.push_str("    SAFE_BRANCH=$(echo \"$CURRENT_BRANCH\" | sed 's/[/\\\\:]/_/g')\n");
    hook_content.push_str("    CONFIG_FILE=\".git/git-pair/config-$SAFE_BRANCH\"\n\n");
    hook_content.push_str("    # Add co-authors from branch-specific config if it exists\n");
    hook_content.push_str("    if [ -f \"$CONFIG_FILE\" ]; then\n");
    hook_content.push_str("      COAUTHORS=$(grep '^Co-authored-by:' \"$CONFIG_FILE\")\n");
    hook_content.push_str("      if [ -n \"$COAUTHORS\" ]; then\n");
    hook_content.push_str("        echo \"\" >> \"$COMMIT_MSG_FILE\"\n");
    hook_content.push_str("        echo \"$COAUTHORS\" >> \"$COMMIT_MSG_FILE\"\n");
    hook_content.push_str("      fi\n");
    hook_content.push_str("    fi\n");
    hook_content.push_str("  fi\n");
    hook_content.push_str("fi\n");
    
    // Write the hook file
    fs::write(&hook_file, hook_content)
        .map_err(|e| format!("Error writing git hook: {}", e))?;
    
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

fn remove_git_hook() -> Result<(), String> {
    let current_dir = env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let hook_file = current_dir.join(".git").join("hooks").join("prepare-commit-msg");
    
    if hook_file.exists() {
        // Check if this is our hook by looking for git-pair signature
        let hook_content = fs::read_to_string(&hook_file)
            .map_err(|e| format!("Error reading hook file: {}", e))?;
        
        if hook_content.contains("git-pair hook") {
            fs::remove_file(&hook_file)
                .map_err(|e| format!("Error removing git hook: {}", e))?;
        }
    }
    
    Ok(())
}

pub fn clear_coauthors() -> Result<String, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;
    
    // Check if git-pair is initialized for this branch
    if !config_file.exists() {
        return Err(format!("git-pair not initialized for branch '{}'. Please run 'git-pair init' first.", branch_name));
    }
    
    // Reset config file to default content
    let default_config = format!("# git-pair configuration file for branch '{}'\n# Co-authors will be listed here\n", branch_name);
    fs::write(&config_file, default_config)
        .map_err(|e| format!("Error clearing config file: {}", e))?;
    
    // Remove git hook
    remove_git_hook()?;
    
    Ok(format!("Cleared all co-authors for branch '{}' and uninstalled git hook", branch_name))
}

pub fn get_coauthors() -> Result<Vec<String>, String> {
    let config_file = get_branch_config_file()?;
    let branch_name = get_current_branch()?;
    
    if !config_file.exists() {
        return Err(format!("git-pair not initialized for branch '{}'. Please run 'git-pair init' first.", branch_name));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    // Test helper to create a temporary git repository
    fn setup_test_repo() -> std::io::Result<tempfile::TempDir> {
        use std::process::Command;
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path();
        
        // Change to temp directory
        env::set_current_dir(repo_path)?;
        
        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .output()?;
            
        // Configure git user (required for commits)
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .output()?;
            
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .output()?;
        
        Ok(temp_dir)
    }

    #[test]
    fn test_init_pair_config_success() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        
        let result = init_pair_config().expect("Init should succeed");
        assert!(result.contains("Successfully initialized git-pair for branch"));
        
        // Check that files were created
        assert!(Path::new(".git/git-pair").exists());
        
        // Check branch-specific config file exists (should be config-main for default branch)
        let branch_config = get_branch_config_file().expect("Should get branch config file");
        assert!(branch_config.exists());
        
        // Check config file content
        let config_content = fs::read_to_string(&branch_config).expect("Config file should exist");
        assert!(config_content.contains("# git-pair configuration file for branch"));
    }

    #[test]
    fn test_init_pair_config_already_initialized() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        
        // Initialize once
        init_pair_config().expect("First init should succeed");
        
        // Initialize again
        let result = init_pair_config().expect("Second init should succeed");
        assert!(result.contains("git-pair already initialized"));
    }

    #[test]
    fn test_init_pair_config_not_git_repo() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
        
        let result = init_pair_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not in a git repository"));
    }

    #[test]
    fn test_add_coauthor_success() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        
        let result = add_coauthor("John", "Doe", "john.doe@example.com").expect("Add should succeed");
        assert!(result.contains("Added co-author: John Doe"));
        
        // Check branch-specific config file was updated
        let branch_config = get_branch_config_file().expect("Should get branch config file");
        let config_content = fs::read_to_string(&branch_config).expect("Config file should exist");
        assert!(config_content.contains("Co-authored-by: John Doe <john.doe@example.com>"));
        
        // Check git hook was installed
        assert!(Path::new(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_add_coauthor_duplicate() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        
        // Add first time
        add_coauthor("John", "Doe", "john.doe@example.com").expect("First add should succeed");
        
        // Add same person again
        let result = add_coauthor("John", "Doe", "john.doe@example.com").expect("Duplicate add should succeed");
        assert!(result.contains("already exists"));
    }

    #[test]
    fn test_add_coauthor_not_initialized() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        
        let result = add_coauthor("John", "Doe", "john.doe@example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_multiple_coauthors() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        
        // Add multiple co-authors
        add_coauthor("John", "Doe", "john.doe@example.com").expect("First add should succeed");
        add_coauthor("Jane", "Smith", "jane.smith@example.com").expect("Second add should succeed");
        
        let coauthors = get_coauthors().expect("Get coauthors should succeed");
        assert_eq!(coauthors.len(), 2);
        assert!(coauthors.iter().any(|c| c.contains("John Doe")));
        assert!(coauthors.iter().any(|c| c.contains("Jane Smith")));
    }

    #[test]
    fn test_clear_coauthors_success() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        add_coauthor("John", "Doe", "john.doe@example.com").expect("Add should succeed");
        
        let result = clear_coauthors().expect("Clear should succeed");
        assert!(result.contains("Cleared all co-authors"));
        
        // Check that co-authors were cleared
        let coauthors = get_coauthors().expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());
        
        // Check that git hook was removed
        assert!(!Path::new(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_clear_coauthors_not_initialized() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        
        let result = clear_coauthors();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_get_coauthors_empty() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        
        let coauthors = get_coauthors().expect("Get coauthors should succeed");
        assert!(coauthors.is_empty());
    }

    #[test]
    fn test_get_coauthors_not_initialized() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        
        let result = get_coauthors();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("git-pair not initialized"));
    }

    #[test]
    fn test_git_hook_functionality() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        add_coauthor("Alice", "Johnson", "alice@example.com").expect("Add should succeed");
        
        // Create a test file and commit with -m flag
        fs::write("test.txt", "test content").expect("Should write test file");
        
        Command::new("git")
            .args(&["add", "test.txt"])
            .output()
            .expect("Git add should succeed");
        
        let output = Command::new("git")
            .args(&["commit", "-m", "Test commit message"])
            .output()
            .expect("Git commit should succeed");
        
        assert!(output.status.success());
        
        // Check that the commit message includes co-author
        let log_output = Command::new("git")
            .args(&["log", "--pretty=format:%B", "-1"])
            .output()
            .expect("Git log should succeed");
        
        let commit_message = String::from_utf8(log_output.stdout).expect("Log output should be valid UTF-8");
        assert!(commit_message.contains("Test commit message"));
        assert!(commit_message.contains("Co-authored-by: Alice Johnson <alice@example.com>"));
    }

    #[test]
    fn test_git_config_integration() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        add_coauthor("John", "Doe", "john.doe@example.com").expect("Add should succeed");
        
        // Check that git hook was installed
        assert!(Path::new(".git/hooks/prepare-commit-msg").exists());
        
        let hook_content = fs::read_to_string(".git/hooks/prepare-commit-msg")
            .expect("Hook file should exist");
        assert!(hook_content.contains("git-pair hook"));
        // With per-branch config, co-author names are read dynamically from config files
        // so they won't be hard-coded in the hook
        assert!(hook_content.contains("CURRENT_BRANCH"));
        assert!(hook_content.contains("CONFIG_FILE"));
        
        // Check that the branch-specific config file contains the co-author
        let branch_config = get_branch_config_file().expect("Should get branch config file");
        let config_content = fs::read_to_string(&branch_config).expect("Config file should exist");
        assert!(config_content.contains("John Doe"));
        
        // Clear and check hook was removed
        clear_coauthors().expect("Clear should succeed");
        
        // Hook should be removed
        assert!(!Path::new(".git/hooks/prepare-commit-msg").exists());
    }

    #[test]
    fn test_global_roster_add_and_list() {
        // Use a temporary HOME directory for testing
        let temp_home = tempfile::tempdir().expect("Failed to create temp dir");
        let original_home = env::var("HOME").unwrap_or_default();
        env::set_var("HOME", temp_home.path());
        
        // Test adding to global roster
        let result = add_global_coauthor("alice", "Alice Johnson", "alice@example.com")
            .expect("Should add to global roster");
        assert!(result.contains("Added 'alice'"));
        
        // Test listing global roster
        let roster = get_global_roster().expect("Should get global roster");
        assert_eq!(roster.len(), 1);
        assert_eq!(roster[0], ("alice".to_string(), "Alice Johnson".to_string(), "alice@example.com".to_string()));
        
        // Test duplicate alias
        let result = add_global_coauthor("alice", "Alice Smith", "alice.smith@example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
        
        // Restore original HOME
        env::set_var("HOME", original_home);
    }

    #[test]
    fn test_add_coauthor_from_global() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        let temp_home = tempfile::tempdir().expect("Failed to create temp dir");
        let original_home = env::var("HOME").unwrap_or_default();
        env::set_var("HOME", temp_home.path());
        
        // Initialize branch config
        init_pair_config().expect("Init should succeed");
        
        // Add to global roster
        add_global_coauthor("bob", "Bob Wilson", "bob@example.com")
            .expect("Should add to global roster");
        
        // Test adding from global roster
        let result = add_coauthor_from_global("bob").expect("Should add from global roster");
        assert!(result.contains("Added co-author: Bob Wilson"));
        
        // Verify it was added to branch config
        let coauthors = get_coauthors().expect("Should get coauthors");
        assert_eq!(coauthors.len(), 1);
        assert!(coauthors[0].contains("Bob Wilson"));
        
        // Test non-existent alias
        let result = add_coauthor_from_global("charlie");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in global roster"));
        
        // Restore original HOME
        env::set_var("HOME", original_home);
    }

    #[test]
    fn test_global_roster_empty() {
        let temp_home = tempfile::tempdir().expect("Failed to create temp dir");
        let original_home = env::var("HOME").unwrap_or_default();
        env::set_var("HOME", temp_home.path());
        
        // Test empty global roster
        let roster = get_global_roster().expect("Should get empty global roster");
        assert!(roster.is_empty());
        
        // Restore original HOME
        env::set_var("HOME", original_home);
    }
}
