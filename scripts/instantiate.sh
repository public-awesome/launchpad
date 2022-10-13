
MSG=$(cat <<EOF
{
  "params": {
    "code_id": 249,
    "creation_fee": { "amount": "1000000000", "denom": "ustars" },
    "min_mint_price": { "amount": "50000000", "denom": "ustars" },
    "mint_fee_bps": 1000,
    "max_trading_offset_secs": 86400,
    "extension": {
      "max_token_limit": 10000,
      "max_per_address_limit": 50,
      "airdrop_mint_price": { "amount": "0", "denom": "ustars" },
      "airdrop_mint_fee_bps": 0,
      "shuffle_fee": { "amount": "500000000", "denom": "ustars" }
    }
  }
}

EOF
)

starsd tx wasm instantiate 251  "$MSG"  --label factory --admin stars1paqkeyluuw47pflgwwqaaj8y679zj96aatg5a7 --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443   -b block --yes -o json | jq .

stars14vwfcs5rmvd5l9njnj2l6nz4p4xxv4mxetv9lfzar6l3c6r860xql5pe7u
