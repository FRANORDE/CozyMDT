use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    println!("=== CozyMDT ===");
    println!("  By FRANORDE   ");
    println!("Write a command (or 'exit' to quit):\n");

    loop {
        // Print the prompt
        print!("CozyMDT> ");
        io::stdout().flush().unwrap();

        // Read the user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Launch powershell, connecting its stdin/stdout/stderr
        // directly to CozyMDT's own — this lets interactive commands
        // (like a nested shell, or git commit without -m) work properly
        let status = Command::new("powershell")
            .args(["-Command", input])
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
