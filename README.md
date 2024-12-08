# Axiom

> [!WARNING]
> This project is under active development and makes frequent breaking changes.

## Introduction

**Axiom** is a command-line tool for managing Minecraft servers.

Axiom leverages [PaperMC] as its core server implementation. Since this project
is tailored to my own needs, I intend to keep the project's scope narrow
and focused. I have no current plans to support other server providers.

## Installation

**Requirements**:

| Name | Version | Description |
|:-----|:--------|:------------|
| Java | 20+ | To run the Paper Minecraft server
| tmux | 3.5a+ | To attach/detach from the Minecraft console as needed

### Quickstart

> [!WARNING]
> This software is developed and tested on Linux only.

Install the project using `cargo` by running the following command:

```bash
cargo install axiom --git https://github.com/nicdgonzalez/axiom
```

or download one of the pre-built binaries [here].

### Usage

If you downloaded one of the pre-built binaries, replace `axiom` with the path
to the executable. For example:

```bash
"$(xdg-user-dir DOWNLOAD)/axiom" help
```

or, move the the executable to a directory on path:

```bash
echo "$PATH" | grep -E "(^|:)$HOME/\.local/bin(:|$)" > /dev/null \
    && echo "$HOME/.local/bin exists on PATH" \
    || echo "$HOME/.local/bin does not exist on PATH"

# This directory should be on PATH (XDG Base Directory Specification),
# but sometimes needs to be created manually.
[ ! -e "$HOME/.local/bin" ] \
    && mkdir --parents "$HOME/.local/bin" \
    || echo "$HOME/.local/bin exists!"

# Move the binary from the Downloads directory into `$HOME/.local/bin`.
mv "$(xdg-user-dir DOWNLOAD)/axiom" "$HOME/.local/bin/axiom"
```

Here is a brief overview of some of the current commands.

> [!NOTE]
> Internally, all names provided are normalized to ensure they can be used
> as valid directory names. For example, "My World" becomes "my-world" and both
> can be used interchangeably anywhere that a command requires `name`.

- To create a new server (NOTE: omitting the version defaults to the latest,
  stable build available):

```bash
# The Minecraft version is optional.
axiom new "My World" 1.21.3

# To get the latest, stable release:
axiom new "My World"
```

- To list information on available servers:

```bash
axiom list
```

- To change the version of Minecraft a server is using:

```bash
# To update to the latest version (stable or experimental):
axiom update --allow-experimental my-world

# Because switching to an older version of Minecraft may corrupt your world,
# Axiom will prevent you from downgrading unless you grant it permission:
axiom update --allow-downgrade my-world 1.21.1
```

- To delete a server:

```bash
axiom delete my-world
```

- To create a backup of the server:

```bash
axiom backup new --wait my-world
```

- To see a list of all available commands:

```bash
axiom help
```

## License

Licensed under the [GPL-3.0 License].

[PaperMC]: https://papermc.io/
[here]: https://github.com/nicdgonzalez/axiom/releases
[GPL-3.0 License]: https://www.gnu.org/licenses/gpl-3.0.html
