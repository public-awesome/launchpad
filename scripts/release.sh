#!/bin/bash

version=$(grep "version" ./Cargo.toml | head -1 | cut -d '"' -f 2)
echo "Current version: $version"

echo "Replacing workspace version with $1 in manifest"
sed -i '' "s/version    = .*/version    = \""$1"\"/g" ./Cargo.toml

echo "Publishing packages"
# . ./scripts/publish-packages.sh

echo "Replacing $version with $1 in manifest"
sed -i '' "s/$version/$1/g" ./Cargo.toml

echo "Publishing contracts"
. ./scripts/publish-contracts.sh

echo "Generating schema"
make schema

# hack to fix schema generation
rm -rf contracts/base-factory/schema/raw contracts/base-minter/schema/raw/ contracts/sg-eth-airdrop/schema/raw/ contracts/splits/schema/raw/ contracts/whitelist-immutable/schema/raw/

git tag -a v$1 -m "Release $1"
git push origin v$1
