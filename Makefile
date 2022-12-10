.PHONY: ci-sign deploy-local e2etest e2etest-full lint optimize publish-packages publish-contracts schema

TEST_ADDRS ?= $(shell jq -r '.[].address' ./e2e/configs/test_accounts.json | tr '\n' ' ')


# TODO: Can we delete `ci-sign` command?
ci-sign:
	drone sign public-awesome/launchpad --save

deploy-local:
	docker kill stargaze || true
	docker volume rm -f stargaze_data
	docker run --rm -d --name stargaze \
		-e DENOM=ustars \
		-e CHAINID=testing \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=stargaze_data,target=/root \
		publicawesome/stargaze:7.5.0 /data/entry-point.sh $(TEST_ADDRS)

e2etest:
	RUST_LOG=info CONFIG=configs/cosm-orc.yaml cargo integration-test $(test_name)

e2etest-full: deploy-local optimize e2etest-dev

lint:
	cargo clippy --all-targets -- -D warnings

optimize:
	sh scripts/optimize.sh

publish-packages:
	sh scripts/publish-packages.sh

publish-contracts:
	sh scripts/publish-contracts.sh

schema:
	sh scripts/schema.sh