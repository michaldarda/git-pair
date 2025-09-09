use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "init" => init_pair_config(),
            "add" => {
                if args.len() >= 5 {
                    let name = &args[2];
                    let surname = &args[3];
                    let email = &args[4];
                    add_coauthor(name, surname, email);
                } else {
                    eprintln!("Usage: git-pair add <name> <surname> <email>");
                }
            }
            "clear" => clear_coauthors(),
            _ => {
                println!("Unknown command. Available commands: init, add, clear");
            }
        }
    } else {
        println!("Usage: git-pair <command>");
        println!("Commands:");
        println!("  init              Initialize git-pair in current repository");
        println!("  add <name> <surname> <email>  Add a co-author");
        println!("  clear             Remove all co-authors");
    }
}

fn init_pair_config() {
    match env::current_dir() {
        Ok(current_dir) => {
            println!("Initializing git-pair in: {}", current_dir.display());
            
            // Check if we're in a git repository
            let git_dir = current_dir.join(".git");
            if !git_dir.exists() {
                eprintln!("Error: Not in a git repository. Please run 'git init' first.");
                return;
            }
            
            // Create .git/git-pair directory
            let git_pair_dir = git_dir.join("git-pair");
            if let Err(e) = fs::create_dir_all(&git_pair_dir) {
                eprintln!("Error creating git-pair directory: {}", e);
                return;
            }
            
            // Create config file
            let config_file = git_pair_dir.join("config");
            let default_config = "# git-pair configuration file\n# Co-authors will be listed here\n";
            
            if config_file.exists() {
                println!("git-pair already initialized in this repository");
            } else {
                match fs::write(&config_file, default_config) {
                    Ok(_) => {
                        println!("Successfully initialized git-pair!");
                        println!("Configuration file created at: {}", config_file.display());
                    }
                    Err(e) => eprintln!("Error creating config file: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

fn add_coauthor(name: &str, surname: &str, email: &str) {
    match env::current_dir() {
        Ok(current_dir) => {
            let git_dir = current_dir.join(".git");
            let git_pair_dir = git_dir.join("git-pair");
            let config_file = git_pair_dir.join("config");
            
            // Check if git-pair is initialized
            if !config_file.exists() {
                eprintln!("Error: git-pair not initialized. Please run 'git-pair init' first.");
                return;
            }
            
            // Read existing config
            let existing_content = match fs::read_to_string(&config_file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading config file: {}", e);
                    return;
                }
            };
            
            // Create the co-author entry
            let full_name = format!("{} {}", name, surname);
            let coauthor_line = format!("Co-authored-by: {} <{}>\n", full_name, email);
            
            // Check if this co-author already exists
            if existing_content.contains(&coauthor_line.trim()) {
                println!("Co-author '{}' <{}> already exists", full_name, email);
                return;
            }
            
            // Append the new co-author
            let new_content = existing_content + &coauthor_line;
            
            match fs::write(&config_file, new_content) {
                Ok(_) => {
                    println!("Added co-author: {} <{}>", full_name, email);
                    // Update the git commit template
                    update_commit_template();
                }
                Err(e) => eprintln!("Error writing to config file: {}", e),
            }
        }
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

fn update_commit_template() {
    match env::current_dir() {
        Ok(current_dir) => {
            let git_dir = current_dir.join(".git");
            let git_pair_dir = git_dir.join("git-pair");
            let config_file = git_pair_dir.join("config");
            let template_file = git_pair_dir.join("commit-template");
            
            // Read the config file to get co-authors
            let config_content = match fs::read_to_string(&config_file) {
                Ok(content) => content,
                Err(_) => return,
            };
            
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
                return;
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
            if let Err(e) = fs::write(&template_file, template_content) {
                eprintln!("Error writing commit template: {}", e);
                return;
            }
            
            // Set git to use this template
            let template_path = template_file.to_string_lossy();
            let output = Command::new("git")
                .args(&["config", "commit.template", &template_path])
                .output();
                
            match output {
                Ok(_) => println!("Updated commit template with current co-authors"),
                Err(e) => eprintln!("Error setting git commit template: {}", e),
            }
        }
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

fn clear_coauthors() {
    match env::current_dir() {
        Ok(current_dir) => {
            let git_dir = current_dir.join(".git");
            let git_pair_dir = git_dir.join("git-pair");
            let config_file = git_pair_dir.join("config");
            let template_file = git_pair_dir.join("commit-template");
            
            // Check if git-pair is initialized
            if !config_file.exists() {
                eprintln!("Error: git-pair not initialized. Please run 'git-pair init' first.");
                return;
            }
            
            // Reset config file to default content
            let default_config = "# git-pair configuration file
# Co-authors will be listed here
";
            match fs::write(&config_file, default_config) {
                Ok(_) => {
                    println!("Cleared all co-authors");
                    
                    // Remove commit template file
                    let _ = fs::remove_file(&template_file);
                    
                    // Unset git commit template
                    let output = Command::new("git")
                        .args(&["config", "--unset", "commit.template"])
                        .output();
                        
                    match output {
                        Ok(_) => println!("Removed commit template"),
                        Err(e) => eprintln!("Warning: Could not unset commit template: {}", e),
                    }
                }
                Err(e) => eprintln!("Error clearing config file: {}", e),
            }
        }
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}
