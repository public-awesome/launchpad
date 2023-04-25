ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
FACTORY_CODE_ID=2046
MINTER_CODE_ID=2047

MSG=$(cat <<EOF
{
  "params": {
    "code_id": $MINTER_CODE_ID,
    "creation_fee": {"amount": "5000000000", "denom": "ustars"},
    "min_mint_price": {"amount": "10000000", "denom": "ustars"},
    "mint_fee_bps": 500,
    "max_trading_offset_secs": 604800,
    "extension": {
        "max_token_limit": 10000,
        "max_per_address_limit": 50,
        "airdrop_mint_price": { "denom": "ustars", "amount": "0" },
        "airdrop_mint_fee_bps": 10000,
        "shuffle_fee": { "amount": "500000000", "denom": "ustars" }
    }
    }
}
EOF
)
echo $MSG

starsd tx wasm instantiate $FACTORY_CODE_ID "$MSG" --label "Factory-v0.16.0" \
  --admin=$ADMIN --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .