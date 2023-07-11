ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
FACTORY=stars1dfqgqmvw077capmmwcynqrrarrxqje2a48gn8f4f0jn0ntgrc3kqxeaj9k
INCREASE_ROYALTIES_FACTORY_CODE_ID=2672

starsd tx wasm migrate $FACTORY $INCREASE_ROYALTIES_FACTORY_CODE_ID "{}" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .

# test minting after factory migration