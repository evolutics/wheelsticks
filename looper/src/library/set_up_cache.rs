use super::configuration;
use anyhow::Context;
use std::borrow;
use std::fs;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    for folder in [
        &configuration.cache.scripts.folder,
        &configuration.cache.staging.folder,
        &configuration.cache.workbench,
    ] {
        fs::create_dir_all(folder)
            .with_context(|| format!("Unable to create cache folder: {folder:?}"))?;
    }

    for (file, contents) in [
        (
            &configuration.cache.scripts.build,
            include_str!("assets/build.py"),
        ),
        (
            &configuration.cache.scripts.deploy,
            include_str!("assets/deploy.py"),
        ),
        (
            &configuration.cache.scripts.deploy_on_remote,
            include_str!("assets/deploy_on_remote.py"),
        ),
        (
            &configuration.cache.scripts.move_to_next_version,
            include_str!("assets/move_to_next_version.sh"),
        ),
        (
            &configuration.cache.scripts.playbook,
            include_str!("assets/playbook.yaml"),
        ),
        (
            &configuration.cache.scripts.provision,
            include_str!("assets/provision.py"),
        ),
        (
            &configuration.cache.scripts.provision_test,
            include_str!("assets/provision_test.sh"),
        ),
        (
            &configuration.cache.staging.vagrantfile,
            &get_vagrantfile_contents(configuration)?,
        ),
    ] {
        fs::write(file, contents)
            .with_context(|| format!("Unable to write cache file: {file:?}"))?;
    }

    Ok(())
}

fn get_vagrantfile_contents(
    configuration: &configuration::Main,
) -> anyhow::Result<borrow::Cow<str>> {
    Ok(match &configuration.vagrantfile {
        None => borrow::Cow::from(include_str!("assets/Vagrantfile")),
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Unable to read Vagrantfile: {path:?}"))?;
            borrow::Cow::from(contents)
        }
    })
}
