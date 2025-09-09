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

pub fn init_pair_config() -> Result<String, String> {
    let _current_dir = env::current_dir().map_err(|e| format!("Error getting current directory: {}", e))?;
    let git_pair_dir = get_git_pair_dir()?;
    
    // Create .git/git-pair directory
    fs::create_dir_all(&git_pair_dir).map_err(|e| format!("Error creating git-pair directory: {}", e))?;
    
    // Create config file
    let config_file = git_pair_dir.join("config");
    let default_config = "# git-pair configuration file\n# Co-authors will be listed here\n";
    
    if config_file.exists() {
        Ok("git-pair already initialized in this repository".to_string())
    } else {
        fs::write(&config_file, default_config)
            .map_err(|e| format!("Error creating config file: {}", e))?;
        Ok(format!("Successfully initialized git-pair!\nConfiguration file created at: {}", config_file.display()))
    }
}

pub fn add_coauthor(name: &str, surname: &str, email: &str) -> Result<String, String> {
    let git_pair_dir = get_git_pair_dir()?;
    let config_file = git_pair_dir.join("config");
    
    // Check if git-pair is initialized
    if !config_file.exists() {
        return Err("git-pair not initialized. Please run 'git-pair init' first.".to_string());
    }
    
    // Read existing config
    let existing_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;
    
    // Create the co-author entry
    let full_name = format!("{} {}", name, surname);
    let coauthor_line = format!("Co-authored-by: {} <{}>\n", full_name, email);
    
    // Check if this co-author already exists
    if existing_content.contains(&coauthor_line.trim()) {
        return Ok(format!("Co-author '{}' <{}> already exists", full_name, email));
    }
    
    // Append the new co-author
    let new_content = existing_content + &coauthor_line;
    
    fs::write(&config_file, new_content)
        .map_err(|e| format!("Error writing to config file: {}", e))?;
    
    update_commit_template()?;
    Ok(format!("Added co-author: {} <{}>", full_name, email))
}

pub fn update_commit_template() -> Result<(), String> {
    let git_pair_dir = get_git_pair_dir()?;
    let config_file = git_pair_dir.join("config");
    let template_file = git_pair_dir.join("commit-template");
    
    // Read the config file to get co-authors
    let config_content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Error reading config file: {}", e))?;
    
    // Extract co-author lines
    let coauthor_lines: Vec<&str> = config_content
        .lines()
        .filter(|line| line.starts_with("Co-authored-by:"))
        .collect();
    
    if coauthor_lines.is_empty() {
        // No co-authors, remove the commit template
        let _ = fs::remove_file(&template_file);
        let _ = Command::new("git")
            .args(&["config", "--unset", "commit.template"])
            .output();
        return Ok(());
    }
    
    // Create commit template content
    let mut template_content = String::new();
    template_content.push_str("# Enter your commit message above\n");
    template_content.push_str("# Co-authors will be automatically added below:\n");
    template_content.push('\n');
    
    for coauthor in &coauthor_lines {
        template_content.push_str(coauthor);
        template_content.push('\n');
    }
    
    // Write the template file
    fs::write(&template_file, template_content)
        .map_err(|e| format!("Error writing commit template: {}", e))?;
    
    // Set git to use this template
    let template_path = template_file.to_string_lossy();
    Command::new("git")
        .args(&["config", "commit.template", &template_path])
        .output()
        .map_err(|e| format!("Error setting git commit template: {}", e))?;
    
    Ok(())
}

pub fn clear_coauthors() -> Result<String, String> {
    let git_pair_dir = get_git_pair_dir()?;
    let config_file = git_pair_dir.join("config");
    let template_file = git_pair_dir.join("commit-template");
    
    // Check if git-pair is initialized
    if !config_file.exists() {
        return Err("git-pair not initialized. Please run 'git-pair init' first.".to_string());
    }
    
    // Reset config file to default content
    let default_config = "# git-pair configuration file\n# Co-authors will be listed here\n";
    fs::write(&config_file, default_config)
        .map_err(|e| format!("Error clearing config file: {}", e))?;
    
    // Remove commit template file
    let _ = fs::remove_file(&template_file);
    
    // Unset git commit template
    let _ = Command::new("git")
        .args(&["config", "--unset", "commit.template"])
        .output();
    
    Ok("Cleared all co-authors and removed commit template".to_string())
}

pub fn get_coauthors() -> Result<Vec<String>, String> {
    let git_pair_dir = get_git_pair_dir()?;
    let config_file = git_pair_dir.join("config");
    
    if !config_file.exists() {
        return Err("git-pair not initialized. Please run 'git-pair init' first.".to_string());
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
        assert!(result.contains("Successfully initialized git-pair!"));
        
        // Check that files were created
        assert!(Path::new(".git/git-pair").exists());
        assert!(Path::new(".git/git-pair/config").exists());
        
        // Check config file content
        let config_content = fs::read_to_string(".git/git-pair/config").expect("Config file should exist");
        assert!(config_content.contains("# git-pair configuration file"));
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
        
        // Check config file was updated
        let config_content = fs::read_to_string(".git/git-pair/config").expect("Config file should exist");
        assert!(config_content.contains("Co-authored-by: John Doe <john.doe@example.com>"));
        
        // Check commit template was created
        assert!(Path::new(".git/git-pair/commit-template").exists());
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
        
        // Check that commit template was removed
        assert!(!Path::new(".git/git-pair/commit-template").exists());
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
    fn test_commit_template_format() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        add_coauthor("John", "Doe", "john.doe@example.com").expect("Add should succeed");
        
        let template_content = fs::read_to_string(".git/git-pair/commit-template")
            .expect("Template file should exist");
        
        assert!(template_content.contains("# Enter your commit message above"));
        assert!(template_content.contains("# Co-authors will be automatically added below:"));
        assert!(template_content.contains("Co-authored-by: John Doe <john.doe@example.com>"));
    }

    #[test]
    fn test_git_config_integration() {
        let _temp_dir = setup_test_repo().expect("Failed to setup test repo");
        init_pair_config().expect("Init should succeed");
        add_coauthor("John", "Doe", "john.doe@example.com").expect("Add should succeed");
        
        // Check that git config was set
        let output = Command::new("git")
            .args(&["config", "commit.template"])
            .output()
            .expect("Git command should succeed");
        
        let template_path = String::from_utf8(output.stdout).expect("Output should be valid UTF-8");
        assert!(template_path.contains(".git/git-pair/commit-template"));
        
        // Clear and check git config was unset
        clear_coauthors().expect("Clear should succeed");
        
        let output = Command::new("git")
            .args(&["config", "commit.template"])
            .output()
            .expect("Git command should run");
        
        // Should fail because config was unset
        assert!(!output.status.success());
    }
}
