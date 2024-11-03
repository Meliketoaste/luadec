use std::process::Command;

pub fn run_command(command: &str) -> bool {
    match Command::new("bash").args(["-c", command]).status() {
        Ok(o) => o,
        Err(_e) => return false,
    }
    .success()
}
