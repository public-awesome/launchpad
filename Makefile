.PHONY: ci-sign e2etest lint optimize publish schema

ci-sign:
	drone sign public-awesome/launchpad --save

e2etest:
	mkdir -p e2e/contracts
	cp artifacts/*.wasm e2e/contracts
	cd e2e && go test -v

lint:
	cargo clippy --all-targets -- -D warnings

optimize:
	sh scripts/optimize.sh

publish:
	sh scripts/publish.sh

schema:
	sh scripts/schema.sh