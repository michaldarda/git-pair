use git_pair::{init_pair_config, add_coauthor, clear_coauthors, get_coauthors, add_global_coauthor, get_global_roster, add_coauthor_from_global};
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
                if args.len() >= 3 && args[2] == "--global" {
                    // Global add: git pair add --global alice "Alice Johnson" alice@company.com
                    if args.len() >= 6 {
                        let alias = &args[3];
                        let name = &args[4];
                        let email = &args[5];
                        match add_global_coauthor(alias, name, email) {
                            Ok(message) => println!("{}", message),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    } else {
                        eprintln!("Usage: git-pair add --global <alias> <name> <email>");
                    }
                } else if args.len() >= 5 {
                    // Direct add with name, surname, email
                    let name = &args[2];
                    let surname = &args[3];
                    let email = &args[4];
                    match add_coauthor(name, surname, email) {
                        Ok(message) => println!("{}", message),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else if args.len() == 3 {
                    // Quick add from roster using alias
                    let alias = &args[2];
                    match add_coauthor_from_global(alias) {
                        Ok(message) => println!("{}", message),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    eprintln!("Usage: git-pair add <name> <surname> <email>");
                    eprintln!("   or: git-pair add <alias>");
                    eprintln!("   or: git-pair add --global <alias> <name> <email>");
                }
            }
            "clear" => {
                match clear_coauthors() {
                    Ok(message) => println!("{}", message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "status" | "list" => {
                if args.len() >= 3 && args[2] == "--global" {
                    // List global roster
                    match get_global_roster() {
                        Ok(roster) => {
                            if roster.is_empty() {
                                println!("No entries in global roster");
                                println!("Use 'git pair add --global <alias> <name> <email>' to add entries");
                            } else {
                                println!("Global roster:");
                                for (alias, name, email) in roster {
                                    println!("  {} -> {} <{}>", alias, name, email);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    // List current branch co-authors
                    match get_coauthors() {
                        Ok(coauthors) => {
                            if coauthors.is_empty() {
                                println!("No co-authors configured for current branch");
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
            }
            _ => {
                println!("Unknown command. Available commands: init, add, clear, status, list");
            }
        }
    } else {
        println!("Usage: git-pair <command>");
        println!("Commands:");
        println!("  init                                    Initialize git-pair for current branch");
        println!("  add <name> <surname> <email>            Add a co-author to current branch");
        println!("  add <alias>                             Add co-author from global roster");
        println!("  add --global <alias> <name> <email>     Add co-author to global roster");
        println!("  clear                                   Remove all co-authors from current branch");
        println!("  status                                  Show current branch co-authors");
        println!("  list --global                           Show global roster");
        println!("");
        println!("Environment Variables:");
        println!("  GIT_PAIR_ROSTER_FILE                    Override global roster file location");
    }
}
