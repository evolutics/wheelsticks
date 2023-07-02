use anyhow::Context;
use std::env;

#[derive(Debug, PartialEq)]
pub struct Host {
    pub ssh: Option<Ssh>,
    pub url: String,
}

#[derive(Debug, PartialEq)]
pub struct Ssh {
    pub hostname: String,
    pub port: Option<u16>,
    pub user: Option<String>,
}

pub fn get(url_override: Option<String>) -> anyhow::Result<Host> {
    let effective_url = get_effective_url(url_override)?;

    let url = url::Url::parse(&effective_url)
        .with_context(|| format!("Unable to parse Docker host URL {effective_url:?}"))?;

    Ok(Host {
        ssh: (url.scheme() == "ssh").then(|| {
            let username = url.username();
            Ssh {
                hostname: url.host_str().unwrap_or("").into(),
                port: url.port(),
                user: (!username.is_empty()).then(|| username.into()),
            }
        }),
        url: effective_url,
    })
}

const ENVIRONMENT_VARIABLE: &str = "DOCKER_HOST";

fn get_effective_url(url_override: Option<String>) -> anyhow::Result<String> {
    match url_override {
        None => match env::var(ENVIRONMENT_VARIABLE) {
            Err(env::VarError::NotPresent) => Ok("unix:///var/run/docker.sock".into()), // TODO: Fall back to Docker context.
            Err(env::VarError::NotUnicode(url)) => Err(anyhow::anyhow!(
                "Environment variable {ENVIRONMENT_VARIABLE:?} should be Unicode but is {url:?}"
            )),
            Ok(url) => Ok(url),
        },
        Some(url) => Ok(url),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get {
        use super::*;

        #[test]
        fn handles_invalid_url() {
            assert!(get(Some("..".into())).is_err())
        }

        #[test]
        fn handles_ssh_url() -> anyhow::Result<()> {
            assert_eq!(
                get(Some("ssh://abc@example.com:123".into()))?,
                Host {
                    ssh: Some(Ssh {
                        hostname: "example.com".into(),
                        port: Some(123),
                        user: Some("abc".into()),
                    }),
                    url: "ssh://abc@example.com:123".into(),
                },
            );
            Ok(())
        }

        #[test]
        fn handles_other_url() -> anyhow::Result<()> {
            assert_eq!(
                get(Some("unix:///tmp/a.sock".into()))?,
                Host {
                    ssh: None,
                    url: "unix:///tmp/a.sock".into(),
                },
            );
            Ok(())
        }
    }
}