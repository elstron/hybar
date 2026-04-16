use std::process::Command;
fn needs_shell(cmd: &str) -> bool {
    cmd.contains('|')
        || cmd.contains("&&")
        || cmd.contains("||")
        || cmd.contains('>')
        || cmd.contains('<')
}

fn clean_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|arg| !arg.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn app_lauch(exec: &str) {
    println!("Launching app with exec: {}", exec);
    let exec = exec.trim();

    if exec.is_empty() {
        return;
    }

    if needs_shell(exec) {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("({}) &", exec))
            .current_dir(std::env::var("HOME").unwrap())
            .spawn();
        return;
    }

    let cleaned = clean_exec(exec);

    let parts = match shell_words::split(&cleaned) {
        Ok(p) => p,
        Err(_) => return,
    };

    if parts.is_empty() {
        return;
    }

    let program = &parts[0];
    let args = &parts[1..];
    let full_cmd = if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    };

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("({}) &", full_cmd))
        .spawn();
}
