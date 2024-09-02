package io.github.nicdgonzalez.axiom;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;

import org.bukkit.Bukkit;
import org.bukkit.event.Listener;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.scheduler.BukkitRunnable;

public class Axiom extends JavaPlugin implements Listener {

    /**
     * Whether we are currently listening for external commands.
     */
    private boolean listening = false;
    /**
     * Path to the `.axiom` directory in the user's home directory.
     */
    private static final Path root = Paths.get(System.getProperty("user.home")).resolve(".axiom");

    /**
     * Runs once when the plugin is being loaded onto the server.
     *
     * This method is responsible for setting up and running the loop that
     * continuously listens for incoming external commands.
     */
    @Override
    public void onEnable() {
        Bukkit.getPluginManager().registerEvents(this, this);
        listening = true;
        startListening();
    }

    /**
     * Runs once when the plugin is being unloaded from the server.
     * <p>
     * This method is responsible for closing the loop that listens for external
     * commands.
     */
    @Override
    public void onDisable() {
        listening = false;
    }

    /**
     * Opens a connection for external services to run commands on the server.
     * <p>
     * <strong>Prerequisites</strong>:<br>
     * - Named pipe must be open at `$HOME/.axiom/pipes/<server>`.
     */
    private void startListening() {
        new BukkitRunnable() {

            @Override
            public void run() {
                String serverName = Paths.get(System.getProperty("user.dir")).getFileName().toString();
                Path fifoPath = Axiom.root.resolve(String.format("pipes/%s", serverName));

                if (!fifoPath.toFile().exists()) {
                    Bukkit.getLogger().info(String.format("Pipe not found: %s", fifoPath.toString()));
                    return;
                }

                while (Axiom.this.listening) {
                    try (BufferedReader reader = new BufferedReader(new FileReader(fifoPath.toString()))) {
                        String line;

                        while ((line = reader.readLine()) != null) {
                            final String command = line;
                            // using the Bukkit API in an async task is not allowed
                            runCommandOnMainThread(command);
                        }
                    } catch (IOException e) {
                        Bukkit.getLogger().warning(String.format(
                                "Failed to read external command: %s",
                                e.getMessage()
                        ));
                    }
                }
            }

        }.runTaskAsynchronously(this);
    }

    /**
     * Executes the command on the Minecraft server.
     * <p>
     * This method has direct access to the Minecraft server. The author of the
     * command should have already been authorized prior to this call.
     */
    private void runCommandOnMainThread(String command) {
        new BukkitRunnable() {

            @Override
            public void run() {
                Bukkit.dispatchCommand(Bukkit.getServer().getConsoleSender(), command);
            }

        }.runTask(this);
    }
}
