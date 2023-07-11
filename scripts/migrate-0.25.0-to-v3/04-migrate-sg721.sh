ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
COLLECTION=stars1cftlzh3screymsws4kuhkfx8qup099xdrq0svtug35ga2zpp5f3stacmc8
# COLLECTION=stars13uejaj2529v9z9e8gpcts82zket44j4khkf93vxmtrfn02vpetpskw2mz2
SG721_INCREASE_ROYALTIES_CODE_ID=2674

starsd tx wasm migrate $COLLECTION $SG721_INCREASE_ROYALTIES_CODE_ID "{}" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .