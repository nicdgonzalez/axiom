#!/usr/bin/python

"""
This is a script that downloads a JSON response from the Mojang API that lists
information about every version of Minecraft available, then saves it to a file
for Axiom to use.

We are essentially just caching Mojang's `version_manifest.json` in case
the user is having network connectivity issues, or the user is making multiple
servers in one day and we want to avoid sending multiple requests to the API.

## Getting Started

Requirements:
- Python 3.12+
- crontab

Dependencies:
- requests==2.31

Schedule this script to run once a day.

### Linux

Install `crontab`:

```bash
sudo dnf install cronie cronie-anacron

# This will open a terminal editor to the file where you can define tasks:
crontab -e
```

In the file, add the following line:

```
0 0 * * * /path/to/this/version_manifest.py
```

This script writes logs to `scripts/logs/{__file__}.log`, and crontab executes
the script once when the new task is created, so you can verify that the task
is running by seeing if the log file has been created. If there is no log file,
you may need to reboot your system for crontab to start working.

### Windows

Currently untested.

## Resources

For better visualizing the scheduling on cron tasks:
- https://crontab.cronhub.io/
- https://crontab.guru/
"""

# This script is temporary, until the functionality is applied to the Axiom
# command-line tool. That way the user can also force the version_manifest
# to update in case they are eager to try a new version the day it comes out.

# At that point, the crontab can just be scheduled to update from the CLI tool.
# 0 0 * * * axiom update

import json
import logging
import os
import pathlib

import requests


def setup_logging() -> None:
    # I want each log timestamp to be equal in length, so ensure level names
    # all have a maximum length of 5. Longer names like "WARNING" or "CRITICAL"
    # have shorter names that are equally understood
    logging.addLevelName(logging.WARNING, "WARN")
    logging.addLevelName(logging.CRITICAL, "FATAL")

    formatter = logging.Formatter(
        fmt="%(asctime)s %(levelname)-5s [%(name)s] %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S%z",  # ISO-8601 Format
    )

    logs = pathlib.Path(__file__).parent.joinpath("logs")
    logs.mkdir(exist_ok=True)

    filename, *_ = os.path.basename(__file__).rsplit(".", 2)
    log_file = pathlib.Path(__file__).parent.joinpath("logs", f"{filename}.log")
    file_handler = logging.FileHandler(log_file)
    file_handler.setFormatter(formatter)

    logging.root.addHandler(file_handler)
    logging.root.setLevel(logging.INFO)


def main() -> None:
    setup_logging()
    logging.info("Updating `version_manifest.json`...")

    url = "https://launchermeta.mojang.com/mc/game/version_manifest.json"

    if (response := requests.get(url)).status_code != 200:
        logging.error(
            f"Status code did not return 200 OK: {response.content.decode('utf-8')}"
        )
        return

    (root := pathlib.Path.home().joinpath(".axiom")).mkdir(exist_ok=True)
    version_manifest = root.joinpath("version_manifest.json")

    text = json.dumps(response.json(), skipkeys=True, indent=4)
    _ = version_manifest.write_text(text)

    logging.info(f"Wrote version_manifest to: {version_manifest.as_posix()}")


if __name__ == "__main__":
    main()
