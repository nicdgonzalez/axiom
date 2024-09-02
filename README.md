# Axiom

> [!WARNING]
> This project is under active development and currently considered unstable.

## Introduction

**Axiom** is the foundation for effortless Minecraft server management.

This project provides a collection of tools for managing Minecraft servers.

Axiom leverages [PaperMC](https://papermc.io/) as its core server implementation.
Since this is a project tailored to my own needs, in order to build a solid
foundation and accelerate development, I intend to keep the project's scope
narrow and focused. I have no current plans to support other server providers.

## Installation

### Quickstart

Install the project with `cargo` by running the following command:

```bash
cargo install axiom --git https://github.com/nicdgonzalez/axiom
```

or download one of the pre-built binaries
[here](https://github.com/nicdgonzalez/axiom/releases).

### Recommended

I recommend building the project from source, to ensure it runs on your system.

- Clone the repository:

```bash
git clone --depth 1 -- https://github.com/nicdgonzalez/axiom
cd ./axiom
```

- Run tests

```bash
cargo test
cargo test --doc -- --show-output
```

- Once all tests pass, you're ready to install:

```bash
cargo install --path .
```

## Usage

If you downloaded one of the pre-built binaries, replace `axiom` with the path
to the executable. For example:

```bash
# Windows
%userprofile%\Downloads\axiom.exe help

# Linux
~/Downloads/axiom help
```

- To start a new world:

```bash
# The Minecraft version is optional; if omitted, defaults to the latest release
axiom create "My World" 1.21.1
```

- To list all available worlds:

```console
$ axiom list
Found 1 server:
  1. my-world
```

- When a new version of Minecraft comes out, you can update the server with:

```bash
axiom update "My World"

# or, you can omit the double quotes by using the name provided by `axiom list`
axiom update my-world
```

- To delete a world:

```console
$ axiom delete my-world
Are you sure you want to delete my-world? (y/N): y
my-world has been deleted
```

- Run the following command to see a list of all available commands:

```bash
axiom help
```

For additional documentation, use the `cargo doc` command.

## License

Licensed under the MIT License ([LICENSE](./LICENSE))
