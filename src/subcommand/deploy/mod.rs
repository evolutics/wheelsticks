mod deploy_locally;

use crate::library::command;
use crate::library::compose;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let compose = compose::get(&in_.compose_file)?;

    match in_.ssh_host {
        None => deploy_locally::go(&compose),
        Some(ssh_host) => deploy_remotely(
            &in_.compose_file,
            &compose,
            &Ssh {
                configuration: in_.ssh_configuration,
                host: ssh_host,
                user: in_.ssh_user,
            },
        ),
    }
}

pub struct In {
    pub compose_file: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
}

struct Ssh {
    configuration: Option<path::PathBuf>,
    host: String,
    user: Option<String>,
}

fn deploy_remotely(
    compose_file: &path::Path,
    compose: &compose::Main,
    ssh: &Ssh,
) -> anyhow::Result<()> {
    println!("Assembling artifacts.");
    assemble_artifacts(compose_file, compose)?;
    println!("Synchronizing artifacts.");
    synchronize_artifacts(compose, ssh)?;
    println!("Deploying on remote.");
    run_deploy_on_remote(compose, ssh)
}

fn assemble_artifacts(compose_file: &path::Path, compose: &compose::Main) -> anyhow::Result<()> {
    let source = compose_file;
    let destination = compose.x_wheelsticks.local_workbench.join("compose.yaml");
    let _ = fs::copy(source, &destination)
        .with_context(|| format!("Unable to copy file {source:?} to {destination:?}"))?;
    Ok(())
}

fn synchronize_artifacts(compose: &compose::Main, ssh: &Ssh) -> anyhow::Result<()> {
    let mut source = ffi::OsString::from(&compose.x_wheelsticks.local_workbench);
    source.push("/");
    let source = source;

    let mut destination = ffi::OsString::new();
    if let Some(user) = &ssh.user {
        destination.push(user);
        destination.push("@");
    }
    destination.push(&ssh.host);
    destination.push(":");
    destination.push(&compose.x_wheelsticks.remote_workbench);
    let destination = destination;

    command::status_ok(
        process::Command::new("rsync")
            .args(["--archive", "--delete"])
            .args(
                ssh.configuration.iter().flat_map(|configuration| {
                    ["--rsh".into(), format!("ssh -F {configuration:?}")]
                }),
            )
            .arg("--")
            .args([source, destination]),
    )
}

fn run_deploy_on_remote(compose: &compose::Main, ssh: &Ssh) -> anyhow::Result<()> {
    command::status_ok(
        process::Command::new("ssh")
            .args(
                ssh.configuration
                    .iter()
                    .flat_map(|configuration| [ffi::OsStr::new("-F"), configuration.as_os_str()]),
            )
            .args(ssh.user.iter().flat_map(|user| ["-l", user]))
            .args([&ssh.host, "--", "wheelsticks", "deploy", "--compose-file"])
            .arg(compose.x_wheelsticks.remote_workbench.join("compose.yaml")),
    )
}
