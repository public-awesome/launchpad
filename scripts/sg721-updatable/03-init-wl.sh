# starsd config node $NODE
# starsd config chain-id $CHAIN_ID
# starsd config output json

KEY=$(starsd keys show $ADMIN | jq -r .name)
WL_CODE_ID=1980

TIME=$(date -v+30S +%s)
ENDTIME=$(date -v+3000S +%s)
MSG=$(cat <<EOF
{
  "members": ["stars1cfudsnwnfezvqjnlhtxhssvzneykysc89ad3nm"],
  "start_time": "$(echo $TIME)000000000",
  "end_time": "$(echo $ENDTIME)000000000",
  "mint_price": {
    "amount": "0",
    "denom": "ustars"
  },
  "per_address_limit": 3,
  "member_limit": 10,
  "admins": [],
  "admins_mutable": true
}
EOF
)
echo $MSG


starsd tx wasm instantiate $WL_CODE_ID "$MSG" --label "ZeroMintFeeWhitelist" --amount 100000000ustars \
  --no-admin --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $KEY -y -b block -o json | jq .
