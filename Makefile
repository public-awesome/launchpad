.PHONY: e2etest optimize

optimize:
	sh scripts/optimize.sh

e2etest:
	mkdir -p e2e/contracts
	cp artifacts/*.wasm e2e/contracts
	cd e2e && go test -v

ci-sign:
	drone sign public-awesome/launchpad --save

schema:
	cd contracts/base-factory && cargo schema
	cd contracts/base-minter && cargo schema
	cd contracts/sg721-base && cargo schema
	cd contracts/sg721-metadata-onchain && cargo schema
	cd contracts/sg721-nt && cargo schema
	cd contracts/vending-factory && cargo schema
	cd contracts/vending-minter && cargo schema
	cd ts && yarn codegen
