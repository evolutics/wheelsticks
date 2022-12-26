#!/usr/bin/env python3

import datetime
import os
import pathlib
import subprocess
import time


def main():
    print("Doing provisioning.")
    _do_provisioning()
    print("Rebooting.")
    _reboot()
    print("Testing provisioning.")
    _test_provisioning()


def _do_provisioning():
    scripts_folder = pathlib.Path(os.environ["KEREK_CACHE_SCRIPTS"])
    # TODO: Escape quotes.
    quoted_ssh_configuration = f"'{os.environ['KEREK_SSH_CONFIGURATION']}'"

    subprocess.run(
        [
            "ansible-playbook",
            "--inventory",
            scripts_folder / "inventory.yaml",
            "--ssh-common-args",
            f"-F {quoted_ssh_configuration}",
            "--",
            scripts_folder / "playbook.yaml",
        ],
        check=True,
    )


def _reboot():
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            "-f",
            os.environ["KEREK_SSH_HOST"],
            "--",
            "sudo",
            "reboot",
        ],
        check=True,
    )


def _test_provisioning():
    timeout = datetime.timedelta(seconds=5)

    while True:
        try:
            return _try_to_test_provisioning(timeout)
        except subprocess.CalledProcessError:
            pass
        except subprocess.TimeoutExpired:
            timeout *= 2
        time.sleep(datetime.timedelta(seconds=1).total_seconds())


def _try_to_test_provisioning(timeout):
    scripts_folder = pathlib.Path(os.environ["KEREK_CACHE_SCRIPTS"])
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            os.environ["KEREK_SSH_HOST"],
            "--",
            f"KEREK_DEPLOY_USER={os.environ['KEREK_DEPLOY_USER']}",
            "bash",
        ],
        check=True,
        input=(scripts_folder / "provision_test.sh").read_bytes(),
        timeout=timeout.total_seconds(),
    )


if __name__ == "__main__":
    main()
