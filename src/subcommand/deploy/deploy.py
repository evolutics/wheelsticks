#!/usr/bin/env python3

import os
import pathlib
import shlex
import subprocess


def main():
    print("Synchronizing artifacts.")
    _synchronize_artifacts()
    print("Deploying on remote.")
    _deploy_on_remote()


def _synchronize_artifacts():
    destination = (
        f"{os.environ['WHEELSTICKS_DEPLOY_USER']}@{os.environ['WHEELSTICKS_SSH_HOST']}"
    )
    subprocess.run(
        [
            "rsync",
            "--archive",
            "--delete",
            "--rsh",
            shlex.join(["ssh", "-F", os.environ["WHEELSTICKS_SSH_CONFIGURATION"]]),
            "--",
            f"{os.environ['WHEELSTICKS_LOCAL_WORKBENCH']}/",
            f"{destination}:{os.environ['WHEELSTICKS_REMOTE_WORKBENCH']}",
        ],
        check=True,
    )


def _deploy_on_remote():
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["WHEELSTICKS_SSH_CONFIGURATION"],
            "-l",
            os.environ["WHEELSTICKS_DEPLOY_USER"],
            os.environ["WHEELSTICKS_SSH_HOST"],
            "--",
            f"WHEELSTICKS_REMOTE_WORKBENCH={os.environ['WHEELSTICKS_REMOTE_WORKBENCH']}",
            "python3",
        ],
        check=True,
        input=pathlib.Path(os.environ["WHEELSTICKS_DEPLOY_ON_REMOTE"]).read_bytes(),
    )


if __name__ == "__main__":
    main()
