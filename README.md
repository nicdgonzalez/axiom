# Axiom

> [!WARNING]
> This project is under active development and is currently unstable.

## Introduction

**Axiom** is a command-line tool for managing Minecraft servers.

Axiom leverages [PaperMC] as its core server implementation. Since this project
is tailored to my own needs, I intend to keep the project's scope narrow
and focused. I have no current plans to support other server providers.

## Installation

**Requirements**:

- Java 20+
- tmux

### Quickstart

> [!NOTE]
> This software is only developed and tested on Linux.

Install the project using `cargo` by running the following command:

```bash
cargo install axiom --git https://github.com/nicdgonzalez/axiom
```

or download one of the pre-built binaries [here].

### Usage

If you downloaded one of the pre-built binaries, replace `axiom` with the path
to the executable. For example:

```bash
"$(xdg-user-dir DOWNLOAD)/axiom-0.1.0" help
```

Here is a brief overview of some of the current commands.

> [!NOTE]
> Internally, all names provided are normalized to ensure they can be used
> as valid directory names. For example, "My World" becomes "my-world" and both
> can be used interchangeably anywhere that a command requires `name`.

- To create a new server (omitting the version defaults to the latest, stable
  build available):

```bash
# The Minecraft version is optional.
axiom new "My World" 1.21.3

# To get the latest release, whether experimental or not:
axiom new --allow-experimental "My World"
```

- To list information on available servers:

```console
$ axiom list
```

- To change the version of Minecraft a server is using:

```bash
# To update to the latest version, whatever it is:
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

For additional documentation, try the `cargo doc` command.

## License

Licensed under the [GPL-3.0 License].

[PaperMC]: https://papermc.io/
[here]: https://github.com/nicdgonzalez/axiom/releases
[GPL-3.0 License]: https://www.gnu.org/licenses/gpl-3.0.html
