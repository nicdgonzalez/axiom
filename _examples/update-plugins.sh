#!/usr/bin/bash

set -eo pipefail
[[ ! -z "${TRACE+x}" ]] && set -x

main() {
    local server_name="celestia"
    local server_path="$(axiom list 2> /dev/null | grep -F "$server_name" | awk '{print $1}')"
    local plugin_path="$server_path/plugins"

    echo "Updating plugins for the Celestia Minecraft server..." >&2

    # Geyser
    echo -n $'Updating Geyser...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/Geyser-Spigot.jar" \
        -- \
        https://download.geysermc.org/v2/projects/geyser/versions/latest/builds/latest/downloads/spigot
    echo $'\rGeyser has been updated!' >&2

    # Floodgate
    echo -n $'Updating Floodgate...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/floodgate-spigot.jar" \
        -- \
        https://download.geysermc.org/v2/projects/floodgate/versions/latest/builds/latest/downloads/spigot
    echo $'\rFloodgate has been updated!' >&2

    # DiscordSRV
    echo -n $'Updating DiscordSRV...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/DiscordSRV.jar" \
        -- \
        https://get.discordsrv.com/
    echo $'\rDiscordSRV has been updated!' >&2

    # ViaBackwards
    echo -n $'Updating ViaBackwards...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/ViaBackwards.jar" \
        -- \
        "$(curl --silent --show-error --location https://api.github.com/repos/viaversion/viabackwards/releases/latest | jq --raw-output '.assets[-1].browser_download_url')"
    echo $'\rViaBackwards has been updated!' >&2

    # ViaRewind
    echo -n $'Updating ViaRewind...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/ViaRewind.jar" \
        -- \
        "$(curl --silent --show-error --location https://api.github.com/repos/viaversion/viarewind/releases/latest | jq --raw-output '.assets[-1].browser_download_url')"
    echo $'\rViaRewind has been updated!' >&2

    # ViaVersion
    echo -n $'Updating ViaVersion...' >&2
    curl --silent --show-error --location \
        --output "$plugin_path/ViaVersion.jar" \
        -- \
        "$(curl --silent --show-error --location https://api.github.com/repos/viaversion/viaversion/releases/latest | jq --raw-output '.assets[-1].browser_download_url')"
    echo $'\rViaVersion has been updated!' >&2
}

main "$@"
unset -f main
