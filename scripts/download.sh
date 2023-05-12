#!/bin/bash

RELEASE_URL="https://api.github.com/repos/public-awesome/launchpad/releases/tags/v0.16.0"

# Download the release page and extract the download URLs
curl -s "$RELEASE_URL" | jq -r '.assets[] | select(.name | endswith(".wasm")) | .browser_download_url' | wget -qi -