KEY=$(starsd keys show $USER | jq -r .name)
MINTER=stars1wzqzh7v0xx3qvkm54qh0f8lrse63mcnc6h9a9lwhz952ps8swdjqtjrnvm
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