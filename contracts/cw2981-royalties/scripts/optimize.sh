#!/bin/bash

docker run --rm -v "$(pwd)":/code \
	--platform linux/x86_64 \
	--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/rust-optimizer:0.12.6
