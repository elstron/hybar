pub fn app_lauch(app_cmd: &str) {
    let _ = std::process::Command::new("setsid")
        .arg("sh")
        .arg("-c")
        .arg(app_cmd)
        .current_dir(std::env::var("HOME").unwrap())
        .spawn();
}
