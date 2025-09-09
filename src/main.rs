use git_pair::{init_pair_config, add_coauthor, clear_coauthors, get_coauthors};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "init" => {
                match init_pair_config() {
                    Ok(message) => println!("{}", message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "add" => {
                if args.len() >= 5 {
                    let name = &args[2];
                    let surname = &args[3];
                    let email = &args[4];
                    match add_coauthor(name, surname, email) {
                        Ok(message) => println!("{}", message),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    eprintln!("Usage: git-pair add <name> <surname> <email>");
                }
            }
            "clear" => {
                match clear_coauthors() {
                    Ok(message) => println!("{}", message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "status" => {
                match get_coauthors() {
                    Ok(coauthors) => {
                        if coauthors.is_empty() {
                            println!("No co-authors configured");
                        } else {
                            println!("Current co-authors:");
                            for coauthor in coauthors {
                                println!("  {}", coauthor);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            _ => {
                println!("Unknown command. Available commands: init, add, clear, status");
            }
        }
    } else {
        println!("Usage: git-pair <command>");
        println!("Commands:");
        println!("  init                          Initialize git-pair in current repository");
        println!("  add <name> <surname> <email>  Add a co-author");
        println!("  clear                         Remove all co-authors");
        println!("  status                        Show current co-authors");
    }
}
