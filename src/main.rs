use git_pair::{
    add_coauthor, add_coauthor_from_global, add_global_coauthor, clear_coauthors, get_coauthors,
    get_global_roster, init_pair_config, remove_coauthor,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-V" => {
                println!("git-pair {}", env!("CARGO_PKG_VERSION"));
            }
            "--help" | "-h" | "help" => {
                print_help();
            }
            "init" => match init_pair_config() {
                Ok(message) => println!("{}", message),
                Err(e) => eprintln!("Error: {}", e),
            },
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
            "clear" => match clear_coauthors() {
                Ok(message) => println!("{}", message),
                Err(e) => eprintln!("Error: {}", e),
            },
            "remove" => {
                if args.len() >= 3 {
                    let identifier = &args[2];
                    match remove_coauthor(identifier) {
                        Ok(message) => println!("{}", message),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    eprintln!("Usage: git-pair remove <name|email|alias>");
                    eprintln!("Examples:");
                    eprintln!("  git-pair remove \"John Doe\"");
                    eprintln!("  git-pair remove john.doe@example.com");
                    eprintln!("  git-pair remove alice");
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
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Use 'git-pair --help' for usage information.");
            }
        }
    } else {
        print_help();
    }
}

fn print_help() {
    let help_text = format!(
        r#"git-pair {}
A git extension for pair programming with per-branch co-author management

USAGE:
    git-pair <COMMAND>

COMMANDS:
    init                                    Initialize git-pair for current branch
    add <name> <surname> <email>            Add a co-author to current branch
    add <alias>                             Add co-author from global roster
    add --global <alias> <name> <email>     Add co-author to global roster
    remove <name|email|alias>               Remove a specific co-author from current branch
    clear                                   Remove all co-authors from current branch
    status                                  Show current branch co-authors
    list --global                           Show global roster
    help, --help, -h                        Show this help message
    --version, -V                           Show version information

ENVIRONMENT VARIABLES:
    GIT_PAIR_ROSTER_FILE                    Override global roster file location

EXAMPLES:
    git-pair init
    git-pair add John Doe john.doe@company.com
    git-pair add --global alice "Alice Johnson" alice@company.com
    git-pair add alice
    git-pair remove "John Doe"
    git-pair remove john.doe@company.com
    git-pair remove alice
    git-pair status
    git-pair list --global
"#,
        env!("CARGO_PKG_VERSION")
    );

    print!("{}", help_text);
}
