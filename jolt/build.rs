// see https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::{process::{Command, Output, ExitStatus}, os::unix::process::ExitStatusExt};

fn main() {
    println!("cargo:rerun-if-changed=.git/refs/heads");
    // get the hash of the current commit using git
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap_or_else(|_| {
        println!("cargo:warning=Failed to get git hash! Make sure that your local copy is controlled with git. Defaulting to 'unknown'");
        Output {
            status: ExitStatus::from_raw(0),
            stdout: "unknown".as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    });
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}