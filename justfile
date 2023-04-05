# https://github.com/casey/just

# use this also for pi version 1
build-for-pi2:
    #!/usr/bin/env bash
    set -euo pipefail
    export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig"
    cross build --target arm-unknown-linux-gnueabihf --release
