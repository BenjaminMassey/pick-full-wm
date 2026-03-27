pub fn run_command(command: &str) {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
    {
        Ok(_) => log::info!("Ran command: \"{}\"", command),
        Err(e) => log::error!("Failed to run command \"{}\": {}", command, e),
    };
}
