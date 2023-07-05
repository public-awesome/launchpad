ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
FACTORY=stars18xydeup6yzht7app47vju0npv9fsy4ea4gy0rudx2xzjh3t3s95sa8snj9
# COLLECTION=stars16w3v5dn5m6nu0d6wnyju4jr78ldg6hys3344n4j6vljakvympueq2d3uaz
INCREASE_ROYALTIES_FACTORY_CODE_ID=2631

starsd tx wasm migrate $FACTORY $INCREASE_ROYALTIES_FACTORY_CODE_ID "{}" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .

# test minting after factory migration