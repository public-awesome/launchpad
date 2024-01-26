#!/bin/bash
set -eux
git checkout -b release-$1
PROJECT_PATH=$(pwd)
version=$(grep "version" ./Cargo.toml | head -1 | cut -d '"' -f 2)
echo "Current version: $version"

echo "Replacing workspace version with $1 in manifest"
sed -i '' "s/version    = .*/version    = \""$1"\"/g" ./Cargo.toml

echo "Replacing $version with $1 in manifest"
cd $PROJECT_PATH
sed -i '' "s/$version/$1/g" ./Cargo.toml

echo "Publishing packages"
. ./scripts/publish-packages.sh

echo "Publishing contracts"
. ./scripts/publish-contracts.sh

echo "Generating schema"
cd $PROJECT_PATH
cargo build
. ./scripts/schema.sh $1

echo "Bump NPM version"
cd $PROJECT_PATH
cd ts
npm version $1

cd $PROJECT_PATH
git commit -am "Release $1"
git push origin release-$1

echo "Push tag to Github"
git tag -a v$1 -m "Release $1"
git push origin v$1

echo "Publish Typescript types to NPM"
cd $PROJECT_PATH
cd ts
npm publish --access public

cd ..
