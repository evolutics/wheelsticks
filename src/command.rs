use anyhow::Context;
use serde::de;
use std::io;
use std::io::Write;
use std::process;
use std::thread;

pub fn piped_ok(
    writer: &mut process::Command,
    reader: &mut process::Command,
) -> anyhow::Result<()> {
    go(
        writer.stdout(process::Stdio::piped()),
        process::Command::spawn,
        |mut child| {
            let stdout = child.stdout.take().context("Unable to open stdout")?;
            status_ok(reader.stdin(stdout))?;
            wait_ok(&mut child)
        },
    )
}

pub fn status_ok(command: &mut process::Command) -> anyhow::Result<()> {
    go(command, process::Command::status, |status| {
        if status.success() {
            Ok(())
        } else {
            status_error(status)
        }
    })
}

pub fn stdin_ok(input: &'static [u8], command: &mut process::Command) -> anyhow::Result<()> {
    go(
        command.stdin(process::Stdio::piped()),
        process::Command::spawn,
        |mut child| {
            let mut stdin = child.stdin.take().context("Unable to open stdin")?;
            thread::spawn(move || stdin.write_all(input).context("Unable to write to stdin"));
            wait_ok(&mut child)
        },
    )
}

pub fn stdout_json<T: de::DeserializeOwned>(command: &mut process::Command) -> anyhow::Result<T> {
    go(
        command.stderr(process::Stdio::inherit()),
        process::Command::output,
        |output| {
            if output.status.success() {
                serde_json::from_slice(&output.stdout)
                    .context("Unable to deserialize JSON from stdout")
            } else {
                status_error(output.status)
            }
        },
    )
}

pub fn stdout_table<const N: usize>(
    command: &mut process::Command,
) -> anyhow::Result<Vec<[String; N]>> {
    go(
        command.stderr(process::Stdio::inherit()),
        process::Command::output,
        |output| {
            if output.status.success() {
                let table =
                    String::from_utf8(output.stdout).context("Stdout is not valid UTF-8")?;
                table
                    .lines()
                    .enumerate()
                    .map(|(row_index, row)| {
                        let fields = row
                            .split_whitespace()
                            .map(|field| field.into())
                            .collect::<Vec<_>>();

                        fields.try_into().map_err(|fields: Vec<_>| {
                            let line_number = row_index + 1;
                            let field_count = fields.len();
                            anyhow::anyhow!(
                                "Unable to parse result line {line_number}, \
                                expected {N} fields \
                                but got {field_count}: {row:?}"
                            )
                        })
                    })
                    .collect()
            } else {
                status_error(output.status)
            }
        },
    )
}

pub fn stdout_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    go(
        command.stderr(process::Stdio::inherit()),
        process::Command::output,
        |output| {
            if output.status.success() {
                String::from_utf8(output.stdout).context("Stdout is not valid UTF-8")
            } else {
                status_error(output.status)
            }
        },
    )
}

fn go<
    F: FnOnce(&mut process::Command) -> io::Result<T>,
    G: FnOnce(T) -> anyhow::Result<U>,
    T,
    U,
>(
    command: &mut process::Command,
    run: F,
    evaluate: G,
) -> anyhow::Result<U> {
    match run(command) {
        Err(error) => Err(anyhow::anyhow!(error))
            .with_context(|| format!("Unable to run command: {command:?}")),
        Ok(value) => evaluate(value)
            .with_context(|| format!("Unable to evaluate result of command: {command:?}")),
    }
}

fn status_error<T>(status: process::ExitStatus) -> anyhow::Result<T> {
    Err(anyhow::anyhow!("{status}"))
}

fn wait_ok(child: &mut process::Child) -> anyhow::Result<()> {
    let status = child.wait().context("Unable to wait")?;

    if status.success() {
        Ok(())
    } else {
        status_error(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case(invalid_program_(), bash("true"), false; "invalid writer")]
    #[test_case::test_case(bash("true"), invalid_program_(), false; "invalid reader")]
    #[test_case::test_case(bash("false"), bash("true"), false; "writer failure")]
    #[test_case::test_case(bash("true"), bash("false"), false; "reader failure")]
    #[test_case::test_case(bash("echo 'Hi'"), bash("[[ $(cat) == 'Hi' ]]"), true; "success")]
    fn piped_ok_handles(
        mut writer: process::Command,
        mut reader: process::Command,
        expected: bool,
    ) {
        assert_eq!(piped_ok(&mut writer, &mut reader).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(bash("true"), true; "success")]
    #[test_case::test_case(bash("false"), false; "failure")]
    fn status_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(status_ok(&mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(bash("[[ $(cat) == 'Hi' ]]"), true; "success")]
    #[test_case::test_case(bash("[[ $(cat) != 'Hi' ]]"), false; "failure")]
    fn stdin_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(stdin_ok("Hi".as_bytes(), &mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(bash("echo '\"Hi\"'"), Some("Hi".into()); "success")]
    fn stdout_json_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_json(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(
        bash("printf '13 a  b\n 8 x\tyz'"),
        Some(vec![
            ["13".into(), "a".into(), "b".into()],
            ["8".into(), "x".into(), "yz".into()],
        ]);
        "success"
    )]
    fn stdout_table_handles(mut command: process::Command, expected: Option<Vec<[String; 3]>>) {
        assert_eq!(stdout_table(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(bash("printf 'Hi'"), Some("Hi".into()); "success")]
    fn stdout_utf8_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_utf8(&mut command).ok(), expected)
    }

    fn invalid_program_() -> process::Command {
        process::Command::new("")
    }

    fn bash(script: &str) -> process::Command {
        let mut command = process::Command::new("bash");
        command.args(["-c", script]);
        command
    }
}
