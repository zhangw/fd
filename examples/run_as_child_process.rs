use anyhow::{Error, Result};
use std::{
    env,
    process::{Command, Output},
};

fn parse_fd_output(output: &Output) -> Result<Vec<String>, Error> {
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        Ok(lines.iter().map(|line| line.to_string()).collect())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("fd command failed with error: {}", stderr))
    }
}

fn run_fd(fd_cli_path: &str, pattern: &str, path: &str) -> Result<Vec<String>, Error> {
    let output = Command::new(fd_cli_path)
        .arg("-I")
        .arg("-t")
        .arg("f")
        .arg("--color")
        .arg("never")
        .arg("-g")
        .arg(pattern)
        .arg(path)
        .output();
    match output {
        Ok(output) => parse_fd_output(&output),
        Err(err) => Err(anyhow::anyhow!("failed to run fd command: {}", err)),
    }
}

fn main() {
    let path = env::current_dir().expect("failed to get current directory");
    let path = path.to_str().expect("failed to convert path to string");
    match run_fd("./target/debug/fd", "build.rs", path) {
        Ok(files) => {
            for file in files {
                println!("{}", file);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
