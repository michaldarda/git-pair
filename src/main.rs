use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "init" {
        init_pair_config();
    } else {
        println!("Hello, world!");
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
