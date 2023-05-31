KEY=$(starsd keys show $ADMIN | jq -r .name)
FACTORY_CODE_ID=2352
MINTER_CODE_ID=2353

MSG=$(cat <<EOF
{
  "params": {
    "code_id": $MINTER_CODE_ID,
    "allowed_sg721_code_ids": [2354],
    "frozen": false,
    "creation_fee": {"amount": "5000000000", "denom": "ustars"},
    "min_mint_price": {"amount": "0", "denom": "ustars"},
    "mint_fee_bps": 1000,
    "max_trading_offset_secs": 604800,
    "extension": {
        "max_per_address_limit": 50,
        "airdrop_mint_price": { "denom": "ustars", "amount": "0" },
        "airdrop_mint_fee_bps": 10000,
        "dev_fee_address": "stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd"
    }
  }
}
EOF
)
echo $MSG


starsd tx wasm instantiate $FACTORY_CODE_ID "$MSG" --label "OpenEditionFactory" \
  --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443 \
  --no-admin --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $KEY -y -b block -o json | jq .
