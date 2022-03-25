.PHONY: e2etest optimize
optimize:
	sh scripts/optimize.sh
e2etest:
	mkdir -p e2e/contracts
	cp artifacts/*.wasm e2e/contracts
	cd e2e && go test -v
