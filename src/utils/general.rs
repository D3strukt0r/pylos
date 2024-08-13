use anyhow::Result;
use bollard::Docker;
use clap::{Parser, Subcommand};
use std::{collections::HashMap, env, path::Path};
use bollard::network::{CreateNetworkOptions, ListNetworksOptions};

use crate::{CONFIG_FILE_NAME_LOCAL, CONFIG_FILE_NAME_PROJECT};

use super::{app_config::AppConfig, path::find_recursively};

#[derive(Debug, Parser)]
#[command(version, about = "A CLI for managing local Docker development environments", long_about = None)]
pub struct Cli {
    /// The name of the service to run the command in. If omitted, the first service in the project will be used.
    #[arg(short, long)]
    pub service: Option<String>,

    /// Run the command in offline mode. This will prevent dev-cli from trying to connect to the internet.
    #[arg(short, long)]
    pub offline: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    pub exec_command: Vec<String>,
}

#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum Commands {
    /// Initialize a new project for dev-cli using pre-defined templates
    Init,
    /// Starts a docker compose project
    Start,
    /// Stop and remove the containers of a project. Does not lose or harm anything unless you add --remove-data.
    Stop {
        #[arg(long, default_value("false"))]
        remove_data: bool,
    },
    /// Stops, removes and starts a project again
    Restart,
    /// Stop all projects and dev-cli containers (Traefik, etc.)
    Poweroff,
    /// Execute a shell command in the container for a service.
    Exec {
        #[arg(short, long)]
        service: Option<String>,

        #[arg(short, long)]
        user: Option<String>,

        command: Vec<String>,
    },
    /// Run a command defined in the config file
    Run {
        command: Vec<String>,
    },
    /// Starts a shell session in the container for a service
    Shell,
    /// Launches the default URL in the default browser
    Launch,
    /// Show the status of the containers of this project
    Status,
    /// Show the status of all projects that ran through dev-cli
    GlobalStatus,


    // Removes items dev-cli has created
    //Clean,
    // Generate the autocompletion script for the specified shell
    //Completion,
    // Create or modify a dev-cli project configuration in the current directory
    //Config,
    // Remove all project information (including database) for an existing project
    //Delete,
    // Get a detailed description of a running dev-cli project
    //Describe,
    // Dump a database to a file or to stdout
    //ExportDb,
    // Get/Download a 3rd party add-on (service, provider, etc.)
    //Get,
    // Manage your hostfile entries.
    //Hostname,
    // Import a SQL dump file into the project
    //ImportDb,
    // Pull the uploaded files directory of an existing project to the default public upload directory of your project
    //ImportFiles,
    // List projects
    //List,
    // Get the logs from your running services.
    //Logs,
    // Add or remove, enable or disable extra services
    //Service,
    // Create a database snapshot for one or more projects.
    //Snapshot,
}

impl Commands {
    pub fn requires_docker(&self) -> bool {
        match self {
            Commands::Start
            | Commands::Stop { .. }
            | Commands::Restart
            | Commands::Poweroff
            | Commands::Exec { .. }
            | Commands::Run { .. }
            | Commands::Shell
            | Commands::Status
            | Commands::GlobalStatus => true,
            _ => false
        }
    }
}

pub fn is_docker_required(
    command: &Option<Commands>,
    exec_command: &Vec<String>,
) -> bool {
    let required_by_command = match command {
        Some(command) => command.requires_docker(),
        None => false,
    };
    required_by_command || exec_command.len() > 0
}

pub async fn docker_running(docker: &Docker) -> String {
    match docker.ping().await {
        Ok(result) => result,
        Err(error) => {
            println!("Docker doesn't seem to be turned on ({})", error);
            sysexits::ExitCode::OsErr.exit()
            //Err(anyhow::anyhow!("Docker doesn't seem to be turned on ({})", error))
        }
    }
}

pub fn check_and_setup_system() {
    // Check that Homebrew is installed
    //let command = "brew --version";
    //let homebrew_check = subprocess::Exec::shell(command).capture().unwrap();
    //if !homebrew_check.exit_status.success() {
    //    println!("Homebrew is not installed");
    //    println!("Installing Homebrew...");
    //    let command = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
    //    let homebrew_install = subprocess::Exec::cmd(command).capture().unwrap();
    //    if homebrew_install.exit_status.success() {
    //        println!("Homebrew installed successfully");
    //    } else {
    //        println!("Homebrew installation failed");
    //        std::process::exit(match homebrew_install.exit_status {
    //            subprocess::ExitStatus::Exited(code) => code as i32,
    //            subprocess::ExitStatus::Signaled(code) => code as i32,
    //            subprocess::ExitStatus::Other(code) => code,
    //            subprocess::ExitStatus::Undetermined => 1,
    //        });
    //    }
    //}

    // Check that Docker is installed
    //let command = "docker --version";
    //let docker_check = subprocess::Exec::shell(command).capture().unwrap();
    //if !docker_check.exit_status.success() {
    //    println!("Docker is not installed");
    //    println!("Installing Docker...");
    //    let command = "brew install docker";
    //    let docker_install = subprocess::Exec::shell(command).capture().unwrap();
    //    if docker_install.exit_status.success() {
    //        println!("Docker installed successfully");
    //    } else {
    //        println!("Docker installation failed");
    //        std::process::exit(match docker_install.exit_status {
    //            subprocess::ExitStatus::Exited(code) => code as i32,
    //            subprocess::ExitStatus::Signaled(code) => code as i32,
    //            subprocess::ExitStatus::Other(code) => code,
    //            subprocess::ExitStatus::Undetermined => 1,
    //        });
    //    }
    //}

    // Check that Git is installed
    //let command = "git --version";
    //let git_check = subprocess::Exec::shell(command).capture().unwrap();
    //if !git_check.exit_status.success() {
    //    println!("Git is not installed");
    //    println!("Installing Git...");
    //    let command = "brew install git";
    //    let git_install = subprocess::Exec::shell(command).capture().unwrap();
    //    if git_install.exit_status.success() {
    //        println!("Git installed successfully");
    //    } else {
    //        println!("Git installation failed");
    //        std::process::exit(match git_install.exit_status {
    //            subprocess::ExitStatus::Exited(code) => code as i32,
    //            subprocess::ExitStatus::Signaled(code) => code as i32,
    //            subprocess::ExitStatus::Other(code) => code,
    //            subprocess::ExitStatus::Undetermined => 1,
    //        });
    //    }
    //}

    // Check that jq is installed
    //let command = "jq --version";
    //let jq_check = subprocess::Exec::shell(command).capture().unwrap();
    //if !jq_check.exit_status.success() {
    //    println!("jq is not installed");
    //    println!("Installing jq...");
    //    let command = "brew install jq";
    //    let jq_install = subprocess::Exec::shell(command).capture().unwrap();
    //    if jq_install.exit_status.success() {
    //        println!("jq installed successfully");
    //    } else {
    //        println!("jq installation failed");
    //        std::process::exit(match jq_install.exit_status {
    //            subprocess::ExitStatus::Exited(code) => code as i32,
    //            subprocess::ExitStatus::Signaled(code) => code as i32,
    //            subprocess::ExitStatus::Other(code) => code,
    //            subprocess::ExitStatus::Undetermined => 1,
    //        });
    //    }
    //}
}

pub async fn check_and_setup_docker(docker: &bollard::Docker) {
    // Check that the docker network "dev-cli-web" exists using bollard
    let mut list_networks_filters = HashMap::new();
    list_networks_filters.insert("name", vec!["dev-cli-web"]);
    let config = ListNetworksOptions {
        filters: list_networks_filters,
    };
    let networks = docker.list_networks(Some(config)).await;

    match networks {
        Ok(networks) => {
            if networks.is_empty() {
                println!("Creating the network 'dev-cli-web'...");
                let config = CreateNetworkOptions {
                    name: "dev-cli-web",
                    ..Default::default()
                };

                let network_created = docker.create_network(config).await;
                match network_created {
                    Ok(_) => println!("Network 'dev-cli-web' created successfully"),
                    Err(error) => {
                        println!("Could not create the network 'dev-cli-web': {}", error);
                        sysexits::ExitCode::OsErr.exit()
                    }
                }
            }
        }
        Err(error) => {
            println!("Could not list networks: {}", error);
            sysexits::ExitCode::OsErr.exit()
        }
    }
}

pub fn get_project_root() -> Result<Box<Path>> {
    let cwd = env::current_dir()?;

    let local_config = find_recursively(&cwd, CONFIG_FILE_NAME_LOCAL);
    let project_config = find_recursively(&cwd, CONFIG_FILE_NAME_PROJECT);

    let project_root = match (local_config.as_ref(), project_config.as_ref()) {
        (Some(filepath), _) => filepath.parent().unwrap(),
        (_, Some(filepath)) => filepath.parent().unwrap(),
        (None, None) => {
            eprintln!("Could not find a project root. Please add a {} or {} to your project root",
                      CONFIG_FILE_NAME_LOCAL, CONFIG_FILE_NAME_PROJECT
            );
            std::process::exit(sysexits::ExitCode::OsErr as i32)
        }
    };

    Ok(Box::from(project_root))
}

pub fn get_app_config(project_root: &Path) -> Result<AppConfig> {
    AppConfig::merge_from_project_root(&project_root)
}
