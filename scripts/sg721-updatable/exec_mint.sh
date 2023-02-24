KEY=$(starsd keys show $USER | jq -r .name)
MSG=$(cat <<EOF
{
    "mint": {}
}
EOF
)

echo $MSG

starsd tx wasm execute $MINTER "$MSG" --amount 50000000ustars \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .