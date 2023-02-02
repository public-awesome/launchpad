#!/bin/bash

# replace workspace version with the new version
sed -i '' "s/version    = .*/version    = \""$1"\"/g" ./Cargo.toml

# publish all packages with the new version
. ./scripts/publish-packages.sh

# replace each dependency version with the new version
version=$(grep "version" ./Cargo.toml | head -1 | cut -d '"' -f 2)
sed -i '' "s/$version/$1/g" ./Cargo.toml

# publish all contracts with the new version
. ./scripts/publish-contracts.sh

make schema

# hack to fix schema generation
rm -rf contracts/base-factory/schema/raw contracts/base-minter/schema/raw/ contracts/sg-eth-airdrop/schema/raw/ contracts/splits/schema/raw/ contracts/whitelist-immutable/schema/raw/

git tag -a v$1 -m "Release $1"
git push origin v$1
