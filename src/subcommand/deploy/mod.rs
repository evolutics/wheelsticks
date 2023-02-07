use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let configuration = configuration::get(&in_.configuration)?;

    let deploy = tempfile::NamedTempFile::new()?;
    fs::write(&deploy, include_str!("deploy.py")).context("Unable to write file: deploy.py")?;

    let deploy_on_remote = configuration
        .x_wheelsticks
        .local_workbench
        .join("deploy_on_remote.py");
    fs::write(deploy_on_remote, include_str!("deploy_on_remote.py"))
        .context("Unable to write file: deploy_on_remote.py")?;

    // TODO: Support deploying on same machine without SSH.

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(deploy.as_ref())
            .env(
                "WHEELSTICKS_DEPLOY_USER",
                configuration.x_wheelsticks.deploy_user,
            )
            .env(
                "WHEELSTICKS_LOCAL_WORKBENCH",
                configuration.x_wheelsticks.local_workbench,
            )
            .env(
                "WHEELSTICKS_REMOTE_WORKBENCH",
                configuration.x_wheelsticks.remote_workbench,
            )
            .env(
                "WHEELSTICKS_SSH_CONFIGURATION",
                in_.ssh_configuration.unwrap_or_default(),
            )
            .env("WHEELSTICKS_SSH_HOST", in_.ssh_host.unwrap_or_default()),
    )
}

pub struct In {
    pub configuration: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
}
