.PHONY: deploy-local e2etest e2etest-full lint optimize publish-packages publish-contracts schema release

TEST_ADDRS ?= $(shell jq -r '.[].address' ./e2e/configs/test_accounts.json | tr '\n' ' ')
GAS_LIMIT ?= "75000000"

deploy-local:
	docker kill stargaze || true
	docker volume rm -f stargaze_data
	docker run --rm -d --name stargaze \
		-e DENOM=ustars \
		-e CHAINID=testing \
		-e GAS_LIMIT=$(GAS_LIMIT) \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=stargaze_data,target=/root \
		publicawesome/stargaze:8.0.0 /data/entry-point.sh $(TEST_ADDRS)

e2etest:
	RUST_LOG=info CONFIG=configs/cosm-orc.yaml cargo integration-test $(TEST_NAME)

e2etest-full: deploy-local optimize e2etest

lint:
	cargo clippy --all-targets -- -D warnings

optimize:
	# NOTE: On a cache miss, the dockerized workspace-optimizer container
	# is creating these dirs with permissions we cannot use in CI.
	# So, we need to ensure these dirs are created before calling optimize.sh:
	mkdir -p artifacts target
	sh scripts/optimize.sh

publish-packages:
	sh scripts/publish-packages.sh

publish-contracts:
	sh scripts/publish-contracts.sh

schema:
	sh scripts/schema.sh $(VERSION)

release:
	sh scripts/release.sh $(VERSION)

upload:
	sh scripts/upload.sh