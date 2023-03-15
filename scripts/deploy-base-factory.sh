FACTORY_CODE_ID=1804
MINTER_CODE_ID=1806
MSG=$(cat <<EOF
{
  "params": {
    "code_id": $MINTER_CODE_ID,
    "creation_fee": { "amount": "250000000", "denom": "ustars" },
    "min_mint_price": { "amount": "5000000", "denom": "ustars" },
    "mint_fee_bps": 10000,
    "max_trading_offset_secs": 0,
    "extension": {}
  }
}

EOF
)

starsd tx wasm instantiate $FACTORY_CODE_ID  "$MSG"  --label "base factory" --no-admin \
  --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto \
  --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443 \
  -b block -o json | jq .


