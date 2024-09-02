# Axiom

> [!NOTE]
> This plugin is designed for use within the Axiom project only.

## Introduction

The **Axiom** plugin enables forwarding server commands from outside
the server console, providing a cross-platform solution that eliminates
the need to manage the server’s startup process directly. With Axiom, I can
start a server in the background, and send commands to it without the console.

This replaces the functionality previously handled by `tmux`, allowing me
to drop that dependency, which typically complicates the setup on Windows.

## Installing

This is typically handled automatically for you.

To build the plugin manually:

```bash
# Linux
./gradlew clean build
cp ./build/libs/Axiom-0.1.0.jar ~/.axiom/servers/<server>/plugins/Axiom.jar
```

```bat
@REM Windows
.\gradlew.bat clean build
COPY .\build\libs\Axiom.0.1.0.jar %userprofile%\.axiom\servers\<server>\plugins\Axiom.jar
```
