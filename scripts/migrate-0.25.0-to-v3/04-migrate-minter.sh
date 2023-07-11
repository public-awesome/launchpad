# does this fix iterate over null (null) error for `mint {}`?

ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
MINTER=stars1ms5z58yrp757sq498dsw35zzq5w9t6zq56jz9xvc8lv488e0e7dsze6eln
# MINTER=stars1d4wn70v5kjtewrqu8tv3cc6hava3zxalwxnvclyqed09lvnahw4sf9cj2r
INCREASE_ROYALTIES_MINTER_CODE_ID=2673

starsd tx wasm migrate $MINTER $INCREASE_ROYALTIES_MINTER_CODE_ID "{}" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .

# test minting after factory migration