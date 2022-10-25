.PHONY: e2etest optimize

ci-sign:
	drone sign public-awesome/launchpad --save

e2etest:
	mkdir -p e2e/contracts
	cp artifacts/*.wasm e2e/contracts
	cd e2e && go test -v

optimize:
	sh scripts/optimize.sh

publish:
	sh scripts/publish.sh

schema:
	sh scripts/schema.sh