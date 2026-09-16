use std::io::{self, Write};
use std::process::{Command, Stdio};

enum Shell {
    PowerShell,
    Cmd,
    Bash,
    CozyT, // built-in shell, handled internally
}

impl Shell {
    fn program_and_flag(&self) -> (&str, &str) {
        match self {
            Shell::PowerShell => ("powershell", "-Command"),
            Shell::Cmd => ("cmd", "/C"),
            Shell::Bash => ("bash", "-c"),
            Shell::CozyT => unreachable!("CozyT has no external program"),
        }
    }

    fn name(&self) -> &str {
        match self {
            Shell::PowerShell => "powershell",
            Shell::Cmd => "cmd",
            Shell::Bash => "bash",
            Shell::CozyT => "cozyt",
        }
    }
}

// Handles a command when CozyT is the active shell.
// Returns true if the command was recognized.
fn handle_cozyt_command(input: &str) -> bool {
    match input {
        "help" => {
            println!("CozyT built-in commands:");
            println!("  help     - show this message");
            println!("  version  - show CozyMDT version");
            println!("  settings  - show CozyMDT settings file");
            true
        }
        "version" => {
            println!("CozyMDT v0.1.0");
            true
        }
        "settings" => {
            match std::env::current_exe() {
                Ok(exe_path) => match exe_path.parent() {
                    Some(exe_dir) => {
                        let mut path = exe_dir.to_path_buf();
                        path.push("settings.jsonc");

                        if !path.exists() {
                            let default_content = "{\n    // CozyMDT settings\n}\n";
                            if let Err(e) = std::fs::write(&path, default_content) {
                                eprintln!("Could not create settings file: {}", e);
                                return true;
                            }
                            println!("Created default settings file at {}", path.display());
                        }

                        if let Err(e) = open::that(&path) {
                            eprintln!("Could not open settings file: {}", e);
                        }
                    }
                    None => {
                        eprintln!("Could not determine executable's folder");
                    }
                },
                Err(e) => {
                    eprintln!("Could not determine executable path: {}", e);
                }
            }
            true
        }
        _ => false,
    }
}

fn main() {
    println!("=== CozyMDT ===");
    println!("  By FRANORDE   ");
    println!("Write a command (or 'exit' to quit):\n");

    let mut current_shell = Shell::PowerShell;

    loop {
        print!("CozyMDT [{}]> ", current_shell.name());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        if let Some(target) = input.strip_prefix("switch ") {
            current_shell = match target.trim() {
                "powershell" | "ps" => Shell::PowerShell,
                "cmd" => Shell::Cmd,
                "bash" => Shell::Bash,
                "cozyt" => Shell::CozyT,
                other => {
                    eprintln!("Unknown shell: {}", other);
                    continue;
                }
            };
            println!("Switched to {}", current_shell.name());
            continue;
        }

        // CozyT doesn't spawn a process — it's handled inline
        if let Shell::CozyT = current_shell {
            if !handle_cozyt_command(input) {
                eprintln!("Unknown CozyT command: '{}' (try 'help')", input);
            }
            continue;
        }

        let (program, flag) = current_shell.program_and_flag();

        let status = Command::new(program)
            .args([flag, input])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .and_then(|mut child| child.wait());

        if let Err(e) = status {
            eprintln!("Error executing command: {}", e);
        }
    }

    println!("Exiting from CozyMDT.");
}
