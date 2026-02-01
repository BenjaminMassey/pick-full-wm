pub fn run_command(command: &str) {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
    {
        Ok(_) => println!("Run command: \"{}\"", command),
        Err(e) => eprintln!("Failed to run command \"{}\": {}", command, e),
    };
}
