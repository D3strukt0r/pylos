use std::env;
use std::path::{Path, PathBuf};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use assert_fs::prelude::*;

#[test]
fn config_golden_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dev-cli")?;
    let config_file = assert_fs::NamedTempFile::new(".dev-cli.yml")?;
    let docker_compose_file = assert_fs::NamedTempFile::new(
        config_file.parent()
            .unwrap()
            .join("compose.yml")
    )?;

    env::set_current_dir(config_file.path().parent().unwrap())?;
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Current working directory: {}", current_dir.display());
    println!(".dev-cli.yml: {}", config_file.path().display());
    println!("compose.yml: {}", docker_compose_file.path().display());

    config_file.write_str("foo: bar\n")?;
    docker_compose_file.write_str("name: foo\n")?;
    cmd.assert()
        .success();
        // .stdout(predicate::str::contains("error"));
    Ok(())
}