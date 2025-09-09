use std::env;
use std::fs;

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
            _ => {
                println!("Unknown command. Available commands: init, add");
            }
        }
    } else {
        println!("Usage: git-pair <command>");
        println!("Commands:");
        println!("  init              Initialize git-pair in current repository");
        println!("  add <name> <surname> <email>  Add a co-author");
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
                }
                Err(e) => eprintln!("Error writing to config file: {}", e),
            }
        }
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}
