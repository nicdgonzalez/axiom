# Axiom

Notes for implementing a **start** command.

## Notes

A message indicating the server started successfully:

```
[16:14:15 INFO]: Done (14.773s)! For help, type "help"
```

<details>
    <summary>
        Full console output
    </summary>

```
[16:14:01 INFO]: [bootstrap] Loading Paper 1.21.5-113-main@ba7fb23 (2025-06-10T23:34:22Z) for Minecraft 1.21.5
[16:14:02 INFO]: [PluginInitializerManager] Initializing plugins...
[16:14:02 INFO]: [PluginInitializerManager] Initialized 0 plugins
[16:14:06 INFO]: Environment: Environment[sessionHost=https://sessionserver.mojang.com, servicesHost=https://api.minec
raftservices.com, name=PROD]
[16:14:06 INFO]: Found new data pack file/bukkit, loading it automatically
[16:14:06 INFO]: Found new data pack paper, loading it automatically
[16:14:07 INFO]: No existing world data, creating new world
[16:14:07 INFO]: Loaded 1373 recipes
[16:14:07 INFO]: Loaded 1484 advancements
[16:14:07 INFO]: Starting minecraft server version 1.21.5
[16:14:07 INFO]: Loading properties
[16:14:07 INFO]: This server is running Paper version 1.21.5-113-main@ba7fb23 (2025-06-10T23:34:22Z) (Implementing API
 version 1.21.5-R0.1-SNAPSHOT)
[16:14:07 INFO]: [spark] This server bundles the spark profiler. For more information please visit https://docs.paperm
c.io/paper/profiling
[16:14:08 INFO]: Server Ping Player Sample Count: 12
[16:14:08 INFO]: Using 4 threads for Netty based IO
[16:14:08 INFO]: [MoonriseCommon] Paper is using 4 worker threads, 1 I/O threads
[16:14:08 INFO]: [ChunkTaskScheduler] Chunk system is using population gen parallelism: true
[16:14:08 INFO]: Default game type: SURVIVAL
[16:14:08 INFO]: Generating keypair
[16:14:08 INFO]: Starting Minecraft server on *:25565
[16:14:08 INFO]: Using epoll channel type
[16:14:08 INFO]: Paper: Using libdeflate (Linux x86_64) compression from Velocity.
[16:14:08 INFO]: Paper: Using OpenSSL 3.x.x (Linux x86_64) cipher from Velocity.
[16:14:08 INFO]: Preparing level "world"
[16:14:12 INFO]: Preparing start region for dimension minecraft:overworld
[16:14:12 INFO]: Preparing spawn area: 2%
[16:14:13 INFO]: Preparing spawn area: 2%
[16:14:13 INFO]: Preparing spawn area: 18%
[16:14:14 INFO]: Preparing spawn area: 53%
[16:14:14 INFO]: Preparing spawn area: 65%
[16:14:14 INFO]: Time elapsed: 2091 ms
[16:14:14 INFO]: Preparing start region for dimension minecraft:the_nether
[16:14:14 INFO]: Preparing spawn area: 2%
[16:14:15 INFO]: Preparing spawn area: 69%
[16:14:15 INFO]: Time elapsed: 523 ms
[16:14:15 INFO]: Preparing start region for dimension minecraft:the_end
[16:14:15 INFO]: Preparing spawn area: 2%
[16:14:15 INFO]: Time elapsed: 136 ms
[16:14:15 INFO]: [spark] Starting background profiler...
[16:14:15 INFO]: Done preparing level "world" (6.719s)
[16:14:15 INFO]: Running delayed init tasks
[16:14:15 INFO]: Done (14.773s)! For help, type "help"
[16:14:15 INFO]: *************************************************************************************
[16:14:15 INFO]: This is the first time you're starting this server.
[16:14:15 INFO]: It's recommended you read our 'Getting Started' documentation for guidance.
[16:14:15 INFO]: View this and more helpful information here: https://docs.papermc.io/paper/next-steps
[16:14:15 INFO]: *************************************************************************************
[16:15:15 INFO]: Server empty for 60 seconds, pausing
>
```

</details>

A message indicating the server did not start successfully:

```
[16:22:13 INFO]: Stopping server
```

<details>
    <summary>
        Full console output
    </summary>

```
[16:22:05 INFO]: [bootstrap] Running Java 21 (OpenJDK 64-Bit Server VM 21.0.7+6; Red Hat, Inc. (Red_Hat-21.0.7.0.6-1)) on Linux 6.14.9-300.fc42.x86_64 (amd64)
[16:22:05 INFO]: [bootstrap] Loading Paper 1.21.5-113-main@ba7fb23 (2025-06-10T23:34:22Z) for Minecraft 1.21.5
[16:22:05 INFO]: [PluginInitializerManager] Initializing plugins...
[16:22:05 INFO]: [PluginInitializerManager] Initialized 0 plugins
[16:22:10 INFO]: Environment: Environment[sessionHost=https://sessionserver.mojang.com, servicesHost=https://api.minecraftservices.com, name=PROD]
[16:22:11 INFO]: Loaded 1373 recipes
[16:22:11 INFO]: Loaded 1484 advancements
[16:22:12 INFO]: Starting minecraft server version 1.21.5
[16:22:12 INFO]: Loading properties
[16:22:12 INFO]: This server is running Paper version 1.21.5-113-main@ba7fb23 (2025-06-10T23:34:22Z) (Implementing API version 1.21.5-R0.1-SNAPSHOT)
[16:22:12 INFO]: [spark] This server bundles the spark profiler. For more information please visit https://docs.papermc.io/paper/profiling
[16:22:12 INFO]: Server Ping Player Sample Count: 12
[16:22:12 INFO]: Using 4 threads for Netty based IO
[16:22:12 INFO]: [MoonriseCommon] Paper is using 4 worker threads, 1 I/O threads
[16:22:12 INFO]: [ChunkTaskScheduler] Chunk system is using population gen parallelism: true
[16:22:13 INFO]: Default game type: SURVIVAL
[16:22:13 INFO]: Generating keypair
[16:22:13 INFO]: Starting Minecraft server on *:25565
[16:22:13 INFO]: Using epoll channel type
[16:22:13 INFO]: Paper: Using libdeflate (Linux x86_64) compression from Velocity.
[16:22:13 INFO]: Paper: Using OpenSSL 3.x.x (Linux x86_64) cipher from Velocity.
[16:22:13 WARN]: **** FAILED TO BIND TO PORT!
[16:22:13 WARN]: The exception was: io.netty.channel.unix.Errors$NativeIoException: bind(..) failed: Address already in use
[16:22:13 WARN]: Perhaps a server is already running on that port?
[16:22:13 ERROR]: Encountered an unexpected exception
java.lang.IllegalStateException: Failed to bind to port
at net.minecraft.server.dedicated.DedicatedServer.initServer(DedicatedServer.java:230) ~[paper-1.21.5.jar:1.21.5-113-ba7fb23]
at net.minecraft.server.MinecraftServer.runServer(MinecraftServer.java:1161) ~[paper-1.21.5.jar:1.21.5-113-ba7fb23]
at net.minecraft.server.MinecraftServer.lambda$spin$2(MinecraftServer.java:308) ~[paper-1.21.5.jar:1.21.5-113-ba7fb23]
at java.base/java.lang.Thread.run(Thread.java:1583) ~[?:?]
Caused by: io.netty.channel.unix.Errors$NativeIoException: bind(..) failed: Address already in use
[16:22:13 ERROR]: This crash report has been saved to: /home/ngonzalez/projects/minecraft/test/server/crash-reports/crash-2025-06-15_16.22.13-server.txt
[16:22:13 INFO]: Stopping server
[16:22:13 INFO]: Saving players
[16:22:13 INFO]: Saving worlds
[16:22:13 INFO]: ThreadedAnvilChunkStorage: All dimensions are saved
[16:22:13 INFO]: Waiting for all RegionFile I/O tasks to complete...
[16:22:13 INFO]: All RegionFile I/O tasks to complete
[16:22:13 INFO]: [MoonriseCommon] Awaiting termination of worker pool for up to 60s...
> 2025-06-15T20:22:13.342024027Z Log4j2-AsyncAppenderEventDispatcher-1-Async WARN Advanced terminal features are not available in this environment
[16:22:13 INFO]: [MoonriseCommon] Awaiting termination of I/O pool for up to 60s...
```

</details>

```bash
cat ~/projects/minecraft/celestia/server/logs/latest.log | tail -n 50 | grep --perl-regexp 'Done \((\d+(\.\d+))?s\)! For help, type "help"'
```
