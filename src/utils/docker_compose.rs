#[derive(Debug)]
pub struct DockerCompose {
    file: std::path::PathBuf,
}

impl DockerCompose {
    pub fn new(file: std::path::PathBuf) -> Self {
        Self {
            file,
        }
    }

    pub fn config(&self) -> Result<Config, serde_yaml::Error> {
        let output_cmd = if cfg!(target_os = "windows") {
            panic!("Windows is not supported yet")
            //std::process::Command::new("cmd")
            //    .args(["/C", "echo hello"])
            //    .output()
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg("docker compose config")
                .current_dir(&self.file.parent().unwrap())
                .output()
        };
        let output = match output_cmd {
            Ok(output) => output,
            Err(error) => {
                println!("Docker doesn't seem to be turned on ({})", error);
                sysexits::ExitCode::OsErr.exit()
            },
        };
        let config_string = match std::str::from_utf8(&output.stdout) {
            Ok(stdout) => stdout,
            Err(error) => {
                println!("Error: {}", error);
                sysexits::ExitCode::OsErr.exit()
            },
        };
        let config = serde_yaml::from_str::<Config>(config_string);
        config
    }

    pub fn exec(&self, service: Option<String>, user: Option<String>, command: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let service_to_exec = match service {
            Some(service) => service,
            None => {
                let config = self.config().unwrap();
                let first_service = config.services.keys().next().unwrap();
                first_service.to_string()
            },
        };

        if cfg!(target_os = "windows") {
            panic!("Windows is not supported yet")
            //std::process::Command::new("cmd")
            //    .args(["/C", "echo hello"])
            //    .output()
        } else {
            let mut cmd = subprocess::Exec::cmd("docker").arg("compose").arg("exec");
            cmd = match user {
                Some(user) => cmd
                    .arg("--user").arg(user),
                None => cmd,
            };
            cmd
                .arg(service_to_exec)
                .args(&command)
                .cwd(&self.file.parent().unwrap())
                .join()?
        };
        Ok(())
    }

    pub fn up(&self, detached: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut extra_args = vec![];

        if detached {
            extra_args.push("--detach");
        }

        let args = vec![vec!["compose", "up"], extra_args].concat();
        let cmd = subprocess::Exec::cmd("docker")
            .args(&args)
            .cwd(&self.file.parent().unwrap())
            .join()?;

        if !cmd.success() {
            println!("Error: {:?}", cmd);
            sysexits::ExitCode::OsErr.exit()
        }
        Ok(())
    }

    pub fn down(&self, remove_volumes: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut extra_args = vec![];

        if remove_volumes {
            extra_args.push("--volumes");
        }

        let args = vec![vec!["compose", "down"], extra_args].concat();
        let cmd = subprocess::Exec::cmd("docker")
            .args(&args)
            .cwd(&self.file.parent().unwrap())
            .join()?;

        if !cmd.success() {
            println!("Error: {:?}", cmd);
            sysexits::ExitCode::OsErr.exit()
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    name: String,
    services: std::collections::BTreeMap<String, Service>,
    networks: Option<std::collections::BTreeMap<String, Network>>,
    volumes: Option<std::collections::BTreeMap<String, Volume>>,
    secrets: Option<std::collections::BTreeMap<String, Secret>>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Service {
    // TODO: Check "https://serde.rs/string-or-struct.html" for how to handle "build"
    //build: Option<String>,
    //command: Option<String>,
    container_name: Option<String>,
    depends_on: Option<std::collections::BTreeMap<String, ServiceDependsOn>>,
    environment: Option<std::collections::BTreeMap<String, String>>,
    image: Option<String>,
    init: Option<bool>,
    labels: Option<std::collections::BTreeMap<String, String>>,
    networks: Option<std::collections::BTreeMap<String, Option<String>>>,
    ports: Option<Vec<ServicePorts>>,
    secrets: Option<Vec<ServiceSecret>>,
    volumes: Option<Vec<ServiceVolume>>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ServiceDependsOn {
    condition: String,
    required: bool
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ServicePorts {
    mode: String,
    target: u16,
    published: String,
    protocol: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ServiceSecret {
    source: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ServiceVolume {
    #[serde(rename = "type")]
    volume_type: String,
    source: String,
    target: String,
    bind: Option<ServiceVolumeBind>,
    // TODO: Don't know the actual type of this
    volume: Option<std::collections::BTreeMap<String, String>>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ServiceVolumeBind {
    create_host_path: bool,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Network {
    name: String,
    external: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Volume {
    name: String,
    driver: Option<String>,
    external: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Secret {
    name: String,
    file: String,
}
