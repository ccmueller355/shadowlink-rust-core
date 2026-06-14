#!/usr/bin/env bash
set -euo pipefail

# ─── Synapse setup for ShadowLink integration tests ─────────────────────
# Uses plain `docker` (no compose required).
#
# Usage:  bash scripts/setup-synapse.sh [start|stop|clean]
#   start   — generate config, inject registration secret, boot Synapse
#   stop    — docker stop + rm the container
#   clean   — stop + remove the persisted data volume

CONTAINER_NAME="shadowlink-synapse-test"
VOLUME_NAME="synapse-data"
REGISTRATION_SECRET="shadowlink-test-secret"

start() {
    echo "==> Generating Synapse config..."
    docker run --rm \
        -e SYNAPSE_SERVER_NAME=localhost \
        -e SYNAPSE_REPORT_STATS=no \
        -v "${VOLUME_NAME}:/data" \
        matrixdotorg/synapse:latest generate

    echo "==> Injecting registration_shared_secret..."
    docker run --rm \
        -v "${VOLUME_NAME}:/data" \
        alpine:3.19 sh -c \
        "grep -q 'registration_shared_secret' /data/homeserver.yaml && \
         sed -i 's/^#\?registration_shared_secret:.*/registration_shared_secret: \"${REGISTRATION_SECRET}\"/' /data/homeserver.yaml || \
         echo 'registration_shared_secret: \"${REGISTRATION_SECRET}\"' >> /data/homeserver.yaml"

    echo "==> Starting Synapse..."
    docker run -d \
        --name "${CONTAINER_NAME}" \
        -p 8008:8008 \
        -e SYNAPSE_SERVER_NAME=localhost \
        -e SYNAPSE_REPORT_STATS=no \
        -v "${VOLUME_NAME}:/data" \
        matrixdotorg/synapse:latest

    echo "==> Waiting for Synapse to be ready..."
    for i in $(seq 1 30); do
        if curl -sf http://localhost:8008/_matrix/client/versions >/dev/null 2>&1; then
            echo "==> Synapse is ready at http://localhost:8008"
            exit 0
        fi
        sleep 1
    done
    echo "ERROR: Synapse did not become ready within 30s"
    exit 1
}

stop() {
    echo "==> Stopping Synapse container..."
    docker stop "${CONTAINER_NAME}" 2>/dev/null || true
    docker rm "${CONTAINER_NAME}" 2>/dev/null || true
    echo "Done."
}

clean() {
    stop
    echo "==> Removing data volume..."
    docker volume rm "${VOLUME_NAME}" 2>/dev/null || true
    echo "Done."
}

case "${1:-start}" in
    start) start ;;
    stop)  stop  ;;
    clean) clean ;;
    *)
        echo "Usage: $0 [start|stop|clean]"
        exit 1
        ;;
esac
