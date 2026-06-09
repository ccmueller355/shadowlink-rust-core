#!/usr/bin/env bash
set -euo pipefail

# Fix: inject registration_shared_secret into an already-generated Synapse config.
# Run from the project root:  bash fix-synapse.sh

echo "==> Injecting registration_shared_secret into homeserver.yaml..."
docker run --rm -v synapse-data:/data alpine sed -i 's/^#\?registration_shared_secret:.*/registration_shared_secret: "shadowlink-test-secret"/' /data/homeserver.yaml

echo "==> Restarting Synapse..."
docker restart shadowlink-synapse-test

echo "==> Waiting for Synapse..."
sleep 3
curl -sf http://localhost:8008/_matrix/client/versions >/dev/null && echo "READY" || echo "Check container logs"
