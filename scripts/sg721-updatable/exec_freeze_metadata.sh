KEY=$(starsd keys show $ADMIN | jq -r .name)
MSG=$(cat <<EOF
{
    "freeze_token_metadata": {}
}
EOF
)

echo $MSG

starsd tx wasm execute $SG721 "$MSG" \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .