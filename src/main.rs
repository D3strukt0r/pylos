#[macro_use]
extern crate lazy_static;

mod commands;
mod utils;

use bollard::Docker;
use clap::Parser;
use utils::general::{ensure_proxy_running, get_app_config, get_project_root};
use std::path::{PathBuf, Path};
use crate::utils::general::{Cli, Commands, is_docker_required, docker_running, check_and_setup_system, check_and_setup_docker};

#[allow(unused)]
use assert_cmd::prelude::*; // Add methods on commands
#[allow(unused)]
use predicates::prelude::*;


// Parameters for config
// - docker-compose-path: Path to the docker-compose file (default: {project-root}/compose.yml)
// - build-script-path: Path to the build script (ex.: ./docker/build/build.sh)
// - run-commands: List of commands to run in the container (ex.: [
//    "dev": {commands: [{container: "node", user: "node", command: "yarn run dev"}]},
//    "update": {
//      parallel: true,
//      commands: [
//        {container: "node", user: "node", command: "yarn upgrade"},
//        {container: "php", user: "www-data", command: "composer update"},
//    },
//    "clear-cache": {commands: [{container: "php", user: "www-data, command: "rm -rf var/cache/*"}]},
// ])

// Global constants for config file names
const CONFIG_FILE_NAME_LOCAL: &str = ".dev-cli.yml";
const CONFIG_FILE_NAME_PROJECT: &str = ".dev-cli.dist.yml";

lazy_static! {
    static ref CONFIG_FILE_PATH_GLOBAL: PathBuf = {
        [
            dirs::config_dir().unwrap(),
            PathBuf::from("dev-cli"),
            PathBuf::from(".dev-cli.yml"),
        ]
        .iter()
        .collect()
    };
}

#[tokio::main]
async fn main() -> Result<sysexits::ExitCode, Box<dyn std::error::Error>> {
    // Parse the command line arguments and stop here if there's an error
    let cli = Cli::parse();

    // Check that the system is ready to run the commands
    check_and_setup_system();

    // Connect to Docker
    let docker = Docker::connect_with_local_defaults()?;

    // Check if command is set and requires a docker connection before connecting or if exec_command is set
    if is_docker_required(&cli.command, &cli.exec_command) {
        docker_running(&docker).await;
        check_and_setup_docker(&docker).await;
    }

    println! {"Global config at {}", CONFIG_FILE_PATH_GLOBAL.clone().into_os_string().into_string().unwrap()};

    // Find .dev-cli.yml/.dev-cli.dist.yml in the current directory or any
    // parent directory to determine the project root
    let project_root = get_project_root()?;
    let app_config = get_app_config(&project_root);
    match app_config {
        Ok(conf) => println!("config loaded: {:?}", conf),
        Err(e) => eprintln!("error loading app config: {:?}", e)
    }

    // Find and read the docker `compose.yml` file
    // TODO: Check if command requires knowledge of the compose config
    let docker_compose_config_path = Path::new(project_root.as_ref()).join("compose.yml");
    let docker_compose = if docker_compose_config_path.is_file() {
        utils::docker_compose::DockerCompose::new(docker_compose_config_path)
    } else {
        println!(
            "Could not find a docker compose file in the project root ({})",
            docker_compose_config_path.display()
        );
        sysexits::ExitCode::OsErr.exit()
    };
    let _docker_compose_config = match docker_compose.config() {
        Ok(config) => config,
        Err(error) => {
            println!("Could not read the docker compose file ({})", error);
            sysexits::ExitCode::OsErr.exit()
        }
    };

    //let images = &docker.list_images(Some(bollard::image::ListImagesOptions::<String> {
    //    all: true,
    //    ..Default::default()
    //})).await.unwrap();
    //for image in images {
    //    println!("-> {:?}", image.id);
    //}

    use Commands::*;
    match cli.command {
        Some(command) => {
            match command {
                Exec { service, user, command } => {
                    commands::exec::run(docker_compose, service, user, command.to_vec())?
                }
                Start => {
                    println!("Starting project ...");
                    ensure_proxy_running();
                    docker_compose.up(None, true)?
                }
                Stop { remove_data } => {
                    if remove_data {
                        println!("Stopping with removing data...");
                    } else {
                        println!("Stopping without removing data...");
                    }
                    docker_compose.down(None, remove_data)?
                }
                _ => {
                    println!("Command not implemented yet: {:?}", command);
                    sysexits::ExitCode::OsErr.exit()
                }
            }
        }
        None => {
            commands::exec::run(docker_compose, cli.service.to_owned(), None, cli.exec_command)?;
        }
    }

    Ok(sysexits::ExitCode::Ok)
}

#[test]
fn no_project_root() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("dev-cli")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not find a project root"));

    Ok(())
}
