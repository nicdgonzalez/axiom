# Axiom

> [!WARNING]\
> This project is under active development. Expect frequent, breaking changes.

**Axiom** is a command-line tool for managing Paper Minecraft servers.

## 📦 Installation

**Requirements**:

- Java 20+: For running the PaperMC server JAR.
- tmux 3.5a+: For attaching/detaching from the Minecraft server console.

Install the application using cargo:

```bash
cargo install --git https://github.com/nicdgonzalez/axiom.git
```

## 🚀 Quickstart

```bash
# Create a new package:
axiom new my-server

# Apply changes to the Minecraft server:
axiom build --accept-eula

# Start the server:
axiom start

# See which servers are currently active:
axiom list

# You can connect to the server from Minecraft at: 127.0.0.1:25565

# See who is connected to the server:
axiom status

# Stop the server:
axiom stop
```

## 📖 Overview

Axiom organizes Minecraft servers into a structure known as a "package." A
package contains metadata that allows Axiom to manage the server effectively.

A package is effectively just a wrapper over the inner Minecraft server. If you
ever decide to stop using Axiom, you can take the inner `server` directory and
operate on it like normal.

### Creating a new package

To create a new package, use the `new` command:

```bash
axiom new example
```

If you have an existing Minecraft server, you can wrap it with the package at
the time of its creation using the `--server` and `--jar` command-line options.

```bash
axiom new \
    --server ./path/to/server \
    --jar ./path/to/server/paper-version-build.jar \
    example
```

> [!NOTE] \
> To run any of the package-related subcommands, you must be inside of the
> package.

Now change into the `example` directory:

```bash
cd ./example
```

### Building

To apply changes to the server (e.g., adding/removing command-line arguments,
adding/removing server properties, etc.), use the `build` command:

```bash
axiom build
```

### Updating

To update the server JAR:

```bash
# To update to the latest available version/build:
axiom update

# To update to a specific version:
axiom update <version>

# To update to a specific build:
axiom update <version> <build>
```

> [!NOTE]\
> If the new version is marked as experimental by PaperMC, you need to add the
> `--allow-experimental` flag.
>
> If the new version is older than the current version, you need to add the
> `--allow-downgrade` flag.

### Starting the Minecraft server

To allow players to connect to the Minecraft server, run the `start` command:

```bash
axiom start
```

### Stopping the Minecraft server

To stop the server, disconnecting all players, run the `stop` command:

```bash
axiom stop
```

## License

This project is licensed under the [GPL-3.0 License].

[gpl-3.0 license]: https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
