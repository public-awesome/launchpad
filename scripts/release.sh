#!/bin/bash

# version    = "0.22.7"

sed -i '' `s/version    = "0.22.7"/version    = "$1"/g` ../Cargo.toml

# . publish-packages.sh
