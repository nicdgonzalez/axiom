# Usage

Please follow along to help get yourself familiar with Axiom.

Most commands have the following syntax:

```bash
axiom <COMMAND> [OPTIONS...] [--] <NAME> <ARGS...>
```

## The `new` command

Let's start by creating a new server. To create a new server, use the `new`
command. It has the following syntax:

```bash
axiom new [--allow-experimental | --accept-eula] [--] <NAME> [VERSION]
```

Internally, all names are sanitized and normalized to ensure they 1) are
case-insensitive, and 2) can be used as valid directory names. For example, "My
World" becomes "my-world", and both can be used interchangeably.

The new command creates a new server, downloads the requested Minecraft
version, and generates the initial files. Let's go ahead and create our first
Minecraft server:

```bash
axiom new my-world
```

Note that we did not specify a Minecraft version. By default, Axiom will get
the latest, stable server version available. That is, if the latest version of
Minecraft is 1.21.4, but PaperMC has it marked as experimental, Axiom will
automatically downgrade to 1.21.3, assuming there is a stable version
available.

You can bypass this check with the `--allow-experimental` flag:

```bash
axiom new --allow-experimental -- my-world
```

After the server has been downloaded, Axiom will run it once to generate the
initial files. Towards the end of the process, you'll be prompted to accept the
Minecraft EULA. Once accepted, you will be ready to start your server!

For your convenience, there is a flag to skip this prompt:

```bash
axiom new --accept-eula -- my-world
```

## The `list` command

We can confirm whether the server was created or not by taking a look at the
`list` command. It has the following syntax:

```bash
axiom list [NAME]
```

The `list` command is useful for displaying information about your servers.
Here is an example of the output it might give, and how you can use it to write
scripts for your server:

```admonish tip
Axiom does its best to output messages that target both the user, and
the machine. If you're writing scripts, you should filter out the user-facing
messages by redirecting the error stream to `/dev/null`.
```

```bash
axiom list
# Found 1 server:
# /home/ngonzalez/.local/share/axiom/servers/my-world 1.21.3 my-world

# Redirecting the error stream to filter out user-facing messages:
axiom list 2> /dev/null
# /home/ngonzalez/.local/share/axiom/servers/my-world 1.21.3 my-world

# Tip: If you have more than one server, you can use `grep` to isolate
# the target server. Pass the result to `awk` to get the data you want:
axiom list 2> /dev/null | grep -F 'my-world' | awk '{print $1}'
# /home/ngonzalez/.local/share/axiom/servers/my-world
```

<details>
<summary>Example: Auto-update server plugins</summary>

With this information, we can add plugins to our server programmatically:

```bash
#!/usr/bin/bash
# file: update-plugins.sh

set -eo pipefail
[[ ! -z "${TRACE+x}" ]] && set -x

main() {
    # Download Geyser, a plugin that allows Bedrock players to connect to Java.
    add_to_plugins "Geyser-Spigot.jar" https://download.geysermc.org/v2/projects/geyser/versions/latest/builds/latest/downloads/spigot

    # Add additional plugins...
}

add_to_plugins() {
    declare FILENAME="$1" URL="$2"

    local SERVER_PATH="$(axiom list 2> /dev/null | grep -F 'my-world' | awk '{print $1}')"
    local PLUGINS_PATH="$SERVER_PATH/plugins"

    curl --silent --show-error --location --output "$PLUGINS_PATH/$FILENAME" -- "$URL"
}

main "$@"
```

Don't forget to make the file executable:

```bash
chmod u+x ./update-plugins.sh
```

Using a scheduler like [crontab], you can routinely update the server's
plugins.

```bash
#!/usr/bin/bash
# file: update-crontab.sh

set -eo pipefail
[[ ! -z "${TRACE+x}" ]] && set -x

main() {
    if ! command -v crontab; then
        echo >&2 "error: expected crontab to be installed and on PATH"
        exit 1
    fi

    local TEMP_FILE="/tmp/$USER-crontab"
    # Update plugins every day at 6:00 AM.
    local JOB="0 6 * * * $PWD/update-plugins.sh"

    if grep -F "$JOB" <<< "$(crontab -l)"; then
        exit 0
    fi

    # Paste the contents of the crontab into temporary file.
    crontab -l > "$TEMP_FILE"
    # Add new job to crontab.
    echo "$JOB" >> "$TEMP_FILE"
    # Make our temporary file the new main file.
    crontab "$TEMP_FILE"
    # Delete temporary file.
    unlink "$TEMP_FILE"
}

main "$@"
```

</details>

## The `start` command

To run the server and allow players to connect to it, we use the `start`
command. It has the following syntax:

```bash
axiom start [OPTIONS] [--] <NAME>
```

The start command is preset with a set of flags called [Aikar's flags], a set
of JVM flags designed to improve the performance of the server. It is currently
not possible to override these flags, though I do plan to add support for this.

To test out our new server, run the following command, open Minecraft,
and connect to `localhost:25565` (25565 is Minecraft's default port).

```bash
axiom start my-world
```

## The `stop` command

To stop the server, disconnecting all players, we use the `stop` command.
It has the following syntax:

```bash
axiom stop [OPTIONS] [--] <NAME>
```

<!--

To start an existing server, use the `start` command. It has the following
syntax:

```bash
axiom start [OPTIONS] -- <NAME>
```

Example:

```bash
axiom start my-world
```

## The `stop` command

To stop a running server, use the `stop` command. It has the following syntax:

```bash
axiom stop [OPTIONS] -- <NAME>
```

Example:

```bash
axiom stop my-world
```

## The `update` command

To change the version of Minecraft a server is using, use the `update` command.
It has the following syntax:

```bash
axiom update [OPTIONS] -- <NAME> [VERSION]
```

Example:

```bash
# Same `--allow-experimental` from the create command.
axiom update --allow-experimental -- my-world
```

Because switching to an older version of Minecraft may corrupt your world,
Axiom will prevent you from downgrading unless you grant it permission:

```bash
# Assume my-world is running 1.21.4
axiom update --allow-downgrade -- my-world 1.21.3
```

## The `backup` commands

To create a backup of the world, use the `backup new` command. It has the
following syntax:

```bash
axiom backup new [OPTIONS] -- <NAME>
```

Examples:

```bash
# Note: This operation takes a while.
axiom backup new my-world

# or, if you need to know when it finishes:
axiom backup new --wait my-world
```
-->

[crontab]: https://man7.org/linux/man-pages/man5/crontab.5.html
[Aikar's flags]: https://docs.papermc.io/paper/aikars-flags
