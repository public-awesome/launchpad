# general wasm store tx
ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
starsd tx wasm store vending_factory-yubrew:sg-955.wasm --from $ADMIN \
    --gas-prices 0.025ustars --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id