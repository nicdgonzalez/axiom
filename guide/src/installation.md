# Installation

Before installing Axiom itself, we need to cover some dependencies first.

```admonish warning title="Windows users"
Axiom requires a GNU-like environment, as well as heavily relies on [tmux], a
terminal multiplexer that lets you switch easily between several programs
in one terminal. Windows users *might* be able to run Axiom using either
[Windows Subsystem for Linux] or [Docker], though both of these options are
currently untested.
```

### Java

Axiom leverages [PaperMC] as its core server implementation, which requires
Java 20.

Install the Java Development Kit (JDK) using your favorite package manager:

```bash
# Fedora
sudo dnf install java-20-openjdk

# Feel free to add additional package managers!
# ...
```

or, try installing an executable binary from
[here](https://www.oracle.com/java/technologies/javase/jdk20-archive-downloads.html).

### tmux

Axiom relies heavily on tmux—a terminal multiplexer that lets you manage
several programs in one terminal—to seamlessly attach, detach, and send
commands to the Minecraft server console.

Again, using your favorite package manager:

```bash
# Fedora
sudo dnf install tmux
```

### Axiom

In order to build the `axiom` executable from source, you will first need to
install Rust and Cargo. Follow the instructions on the
[Rust installation page].

Once you have installed Rust, the following command can be used to build and
install Axiom:

```bash
cargo install --git https://github.com/nicdgonzalez/axiom.git axiom-cli
```

or, download one of the pre-built binaries
[here](https://github.com/nicdgonzalez/axiom/releases).

To make it easier to run, place the executable somewhere on `PATH`.

```bash
# Should exist in PATH (according to the XDG Base Directory Specification),
# but sometimes needs to be created manually.
mkdir --parents "$HOME/.local/bin"

# Move the binary from the Downloads directory to "$HOME/.local/bin":
mv "$(xdg-user-dir DOWNLOAD)/axiom" "$HOME/.local/bin/axiom"
```

[papermc]: https://papermc.io/
[rust installation page]: https://www.rust-lang.org/tools/install
